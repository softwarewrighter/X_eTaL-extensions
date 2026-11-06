#!/usr/bin/env python3
"""Observe the saved compiler plan through OpenCL 1.2, without changing it.

Portable to macOS: pass --data and --output explicitly on a remote host.
This observer adds event records; the pinned GPU CLI independently checks it.
"""
import argparse
import ctypes as C
import ctypes.util
import hashlib
import json
import math
from pathlib import Path
import re
import time


def validate(record):
    if record["outputs"] != [32, 16.625]:
        raise ValueError("device output differs from the checked evaluator")
    expected = [("upload", "b_x", 64), ("upload", "b_w", 64), ("upload", "b_l", 32), ("upload", "b_g", 32),
                ("kernel", "k0", 0), ("kernel", "reduce_add_long", 0), ("kernel", "k1", 0), ("kernel", "reduce_add_float", 0),
                ("download", "b_addr", 8), ("download", "b_addr_2", 4)]
    if [(e["kind"], e["name"], e["bytes"]) for e in record["events"]] != expected:
        raise ValueError("incomplete or incorrect transfer/kernel event trace")
    previous = 0
    for e in record["events"]:
        q, s, a, b = (e[k] for k in ("queued_ns", "submitted_ns", "start_ns", "end_ns"))
        if not 0 < q <= s <= a < b or a < previous:
            raise ValueError("invalid in-order profiling timestamps")
        previous = b
    if [e["name"] for e in record["events"] if e["kind"] == "kernel"] != [
            "k0", "reduce_add_long", "k1", "reduce_add_float"]:
        raise ValueError("kernel coverage differs from compiler plan")
    h = record["host_ms"]
    if any(not math.isfinite(v) or v < 0 for v in h.values()) or sum(h[k] for k in ("bind", "upload", "launch", "download")) > h["total"] + 0.01:
        raise ValueError("invalid host phase measurements")


class OpenCL:
    def __init__(self):
        path = ctypes.util.find_library("OpenCL")
        if not path:
            raise RuntimeError("OpenCL loader not found; a real device is required")
        self.lib = C.CDLL(path)
        self.resources = []

    def function(self, name, result, args):
        f = getattr(self.lib, name)
        f.restype, f.argtypes = result, args
        return f

    def call(self, name, args, types):
        code = self.function(name, C.c_int, types)(*args)
        if code != 0:
            raise RuntimeError(f"{name}: OpenCL error {code}")

    def create(self, name, args, types, release):
        err = C.c_int()
        value = self.function(name, C.c_void_p, types + [C.POINTER(C.c_int)])(*args, C.byref(err))
        if err.value or not value:
            raise RuntimeError(f"{name}: OpenCL error {err.value}")
        self.resources.append((release, value))
        return C.c_void_p(value)

    def info(self, device, key, numeric=False):
        n = C.c_size_t()
        self.call("clGetDeviceInfo", [device, key, 0, None, C.byref(n)], [C.c_void_p, C.c_uint, C.c_size_t, C.c_void_p, C.c_void_p])
        buf = C.create_string_buffer(n.value)
        self.call("clGetDeviceInfo", [device, key, n.value, buf, None], [C.c_void_p, C.c_uint, C.c_size_t, C.c_void_p, C.c_void_p])
        return int.from_bytes(buf.raw, "little") if numeric else buf.value.decode()

    def close(self):
        for name, value in reversed(self.resources):
            self.call(name, [C.c_void_p(value)], [C.c_void_p])


def profile(data, device_index):
    cl = OpenCL()
    ptr, uint, size, ulong = C.c_void_p, C.c_uint, C.c_size_t, C.c_ulonglong
    count = uint()
    cl.call("clGetPlatformIDs", [0, None, C.byref(count)], [uint, ptr, ptr])
    platforms = (ptr * count.value)()
    cl.call("clGetPlatformIDs", [count, platforms, None], [uint, ptr, ptr])
    devices = []
    for platform in platforms:
        n = uint()
        code = cl.function("clGetDeviceIDs", C.c_int, [ptr, ulong, uint, ptr, ptr])(platform, 0xFFFFFFFF, 0, None, C.byref(n))
        if code == -1:
            continue
        if code:
            raise RuntimeError(f"clGetDeviceIDs: {code}")
        ds = (ptr * n.value)()
        cl.call("clGetDeviceIDs", [platform, 0xFFFFFFFF, n, ds, None], [ptr, ulong, uint, ptr, ptr])
        devices.extend(ds)
    if not 0 <= device_index < len(devices):
        raise ValueError("required OpenCL device is absent")
    device = ptr(devices[device_index])
    info = {name: cl.info(device, key, numeric) for name, key, numeric in (
        ("name", 0x102B, False), ("vendor", 0x102C, False), ("driver", 0x102D, False),
        ("version", 0x102F, False), ("compute_units", 0x1002, True),
        ("max_work_group", 0x1004, True), ("local_memory_bytes", 0x1023, True),
        ("host_unified_memory", 0x1035, True))}
    source = (data / "rtx-pipeline.cl").read_bytes()
    plan = (data / "rtx-pipeline.plan").read_text()
    xir = (data / "pipeline.xir").read_text()
    # Both saved profiles emit the same pipeline; refuse a changed plan.
    if (data / "m1-pipeline.cl").read_bytes() != source or (data / "m1-pipeline.plan").read_text().splitlines()[1:] != plan.splitlines()[1:]:
        raise ValueError("review device-specific pipeline differences")
    try:
        start = time.perf_counter_ns()
        context = cl.create("clCreateContext", [None, 1, C.byref(device), None, None], [ptr, uint, ptr, ptr, ptr], "clReleaseContext")
        queue = cl.create("clCreateCommandQueue", [context, device, 2], [ptr, ptr, ulong], "clReleaseCommandQueue")
        src = C.c_char_p(source)
        length = size(len(source))
        program = cl.create("clCreateProgramWithSource", [context, 1, C.byref(src), C.byref(length)], [ptr, uint, ptr, ptr], "clReleaseProgram")
        code = cl.function("clBuildProgram", C.c_int, [ptr, uint, ptr, C.c_char_p, ptr, ptr])(program, 1, C.byref(device), b"-cl-std=CL1.2", None, None)
        if code:
            n = size()
            f = cl.function("clGetProgramBuildInfo", C.c_int, [ptr, ptr, uint, size, ptr, ptr])
            f(program, device, 0x1183, 0, None, C.byref(n))
            log = C.create_string_buffer(n.value)
            f(program, device, 0x1183, n, log, None)
            raise RuntimeError(log.value.decode())
        constants = {name: values.split() for name, values in re.findall(r"^%(\w+) = const \w+ \[[^]]*\] (.+)$", xir, re.M)}
        buffers = {}
        host = {}
        outputs = re.findall(r"^  %\w+ from (\w+), read back", plan, re.M)
        for name, ty, length, value in re.findall(r"^  (\w+): (long|float) (\d+) items?, %(\w+)", plan, re.M):
            scalar = C.c_longlong if ty == "long" else C.c_float
            array = (scalar * int(length))
            host[name] = array(*(map(int if ty == "long" else float, constants[value]) if value in constants else []))
            buffers[name] = cl.create("clCreateBuffer", [context, 1, C.sizeof(host[name]), None], [ptr, ulong, size, ptr], "clReleaseMemObject")
        launches = []
        for name, args, glob, local in re.findall(r"^  \d+: (\w+)\(([^)]*)\) global (\d+) local (\d+):", plan, re.M):
            kernel = cl.create("clCreateKernel", [program, name.encode()], [ptr, C.c_char_p], "clReleaseKernel")
            for index, arg in enumerate(args.split(", ")):
                if arg in buffers:
                    v, n = buffers[arg], C.sizeof(ptr)
                elif arg.startswith("local "):
                    v, n = None, int(arg.split()[1])
                else:
                    v, n = C.c_longlong(int(arg)), C.sizeof(C.c_longlong)
                cl.call("clSetKernelArg", [kernel, index, n, C.byref(v) if v is not None else None], [ptr, uint, size, ptr])
            launches.append((name, kernel, int(glob), int(local)))
        if len(buffers) != 8 or len(launches) != 4 or len(outputs) != 2:
            raise ValueError("review changed pipeline coverage")
        setup_ms = (time.perf_counter_ns() - start) / 1e6
        records = []
        for trial in range(3):
            events = []
            begin = time.perf_counter_ns()
            # Explicit host binding copies exactly the constants the runtime uploads.
            bound = {name: type(a).from_buffer_copy(a) for name, a in host.items() if name.removeprefix("b_") in constants}
            bound_ms = (time.perf_counter_ns() - begin) / 1e6
            phase = time.perf_counter_ns()
            for name, a in bound.items():
                event = ptr()
                cl.call("clEnqueueWriteBuffer", [queue, buffers[name], 1, 0, C.sizeof(a), a, 0, None, C.byref(event)], [ptr, ptr, uint, size, size, ptr, uint, ptr, ptr])
                events.append(("upload", name, C.sizeof(a), event))
            upload_ms = (time.perf_counter_ns() - phase) / 1e6
            phase = time.perf_counter_ns()
            for name, kernel, glob, local in launches:
                event = ptr()
                cl.call("clEnqueueNDRangeKernel", [queue, kernel, 1, None, C.byref(size(glob)), C.byref(size(local)), 0, None, C.byref(event)], [ptr, ptr, uint, ptr, ptr, ptr, uint, ptr, ptr])
                events.append(("kernel", name, 0, event))
            cl.call("clFinish", [queue], [ptr])
            launch_ms = (time.perf_counter_ns() - phase) / 1e6
            phase = time.perf_counter_ns()
            for name in outputs:
                event = ptr()
                a = host[name]
                cl.call("clEnqueueReadBuffer", [queue, buffers[name], 1, 0, C.sizeof(a), a, 0, None, C.byref(event)], [ptr, ptr, uint, size, size, ptr, uint, ptr, ptr])
                events.append(("download", name, C.sizeof(a), event))
            download_ms = (time.perf_counter_ns() - phase) / 1e6
            total_ms = (time.perf_counter_ns() - begin) / 1e6
            trace = []
            for kind, name, n, event in events:
                e = {"kind": kind, "name": name, "bytes": n}
                for key, param in zip(("queued_ns", "submitted_ns", "start_ns", "end_ns"), range(0x1280, 0x1284)):
                    value = ulong()
                    cl.call("clGetEventProfilingInfo", [event, param, 8, C.byref(value), None], [ptr, uint, size, ptr, ptr])
                    e[key] = value.value
                trace.append(e)
                cl.call("clReleaseEvent", [event], [ptr])
            record = {"trial": trial, "outputs": [host[n][0] for n in outputs], "events": trace,
                      "host_ms": {"bind": bound_ms, "upload": upload_ms, "launch": launch_ms, "download": download_ms, "total": total_ms}}
            validate(record)
            records.append(record)
        return {"schema": 1, "device": info, "setup_ms": setup_ms, "records": records,
                "method": "independent ctypes observer; same emitted source, buffers, arguments and launch plan; blocking transfers; in-order profiling queue; each trial reuploads inputs; host and device clocks are separate; trials are observations, not a benchmark",
                "artifacts_sha256": {n: hashlib.sha256((data / n).read_bytes()).hexdigest() for n in ("rtx-pipeline.cl", "rtx-pipeline.plan", "pipeline.xir")},
                "observer_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    finally:
        cl.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=Path(__file__).resolve().parents[1] / "demos/gpu-build-data")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--device", type=int, default=0)
    args = parser.parse_args()
    result = profile(args.data, args.device)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(result["device"]["name"], "PASS", len(result["records"]), "checked event traces")
