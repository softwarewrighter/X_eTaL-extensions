// HEX1000 on its Tang Nano 9K board, simulated for the hex-run video
// (scripts/hex1000-sim.sh): X_eTaL-fpga's board top (hex1000-top.v),
// its UART core and the machine (hex1000.v), at FPGA_COMMIT. The text
// (+text=...) is sent over the serial line at 115200 baud, a character
// after another; the run records the signals the video shows in a VCD
// (+vcd=...): the line, each byte received, the keys queued and the one held, K, the R
// latches, O, the digit latches, the machine's registers and its RAM
// (16 words of 16 bits, 4 RAM words each), and a count of machine
// clocks. It ends when the display has been still for 2 ms.
`timescale 1ns/1ps
module tb;
  reg clk = 0, rst_n = 1, uart_rx = 1;
  wire uart_tx, led;
  hex1000_top dut(clk, rst_n, uart_rx, uart_tx, led);
  always #18.5185 clk = ~clk;

  wire valid = dut.rx_valid[0];
  wire [7:0] rxb = dut.rx_byte[7:0];
  wire key_active = dut.key_active;
  wire [5:0] queued = dut.key_count;
  wire [4:0] key_code = dut.key_code;
  wire [4:0] k = dut.keyboard;
  wire [15:0] r = dut.machine_r;
  wire [5:0] o = dut.machine_o;
  wire [39:0] shown = {dut.digits[7], dut.digits[6], dut.digits[5], dut.digits[4],
                       dut.digits[3], dut.digits[2], dut.digits[1], dut.digits[0]};
  // the machine's state registers, in the order of hex1000.xir's states
  wire [3:0] a = dut.machine_core.n1[3:0];
  wire [3:0] y = dut.machine_core.n2[3:0];
  wire [1:0] x = dut.machine_core.n3[1:0];
  wire s = dut.machine_core.n4;
  wire [5:0] pc = dut.machine_core.n5[5:0];
  wire [3:0] pa = dut.machine_core.n6[3:0];
  // RAM word 4i + j in bits 4j + 3 to 4j of ram<i>
  `define RAM(i) wire [15:0] ram``i = {dut.machine_core.mem27[4*i+3][3:0], dut.machine_core.mem27[4*i+2][3:0], dut.machine_core.mem27[4*i+1][3:0], dut.machine_core.mem27[4*i][3:0]};
  `RAM(0) `RAM(1) `RAM(2) `RAM(3) `RAM(4) `RAM(5) `RAM(6) `RAM(7)
  `RAM(8) `RAM(9) `RAM(10) `RAM(11) `RAM(12) `RAM(13) `RAM(14) `RAM(15)

  reg [15:0] cycle = 0;
  always @(posedge dut.machine_clk) if (!dut.reset) cycle <= cycle + 1;

  reg [8*64-1:0] text;
  reg [8*256-1:0] vcd;
  integer n, i, b, still;
  reg [39:0] last;
  task send; input [7:0] c; begin
    uart_rx = 0; #8680;
    for (b = 0; b < 8; b = b + 1) begin uart_rx = c[b]; #8680; end
    uart_rx = 1; #8680;
  end endtask

  initial begin
    if (!$value$plusargs("text=%s", text)) $fatal(1, "no +text=");
    if (!$value$plusargs("vcd=%s", vcd)) $fatal(1, "no +vcd=");
    $dumpfile(vcd);
    $dumpvars(0, uart_rx, valid, rxb, queued, key_active, key_code, k, r, o, shown, a, y, x, s, pc, pa, cycle,
              ram0, ram1, ram2, ram3, ram4, ram5, ram6, ram7, ram8, ram9, ram10, ram11, ram12, ram13, ram14, ram15);
    #300000;
    n = 64; while (n > 0 && text[8*n-1 -: 8] == 0) n = n - 1;
    for (i = n; i > 0; i = i - 1) send(text[8*i-1 -: 8]);
    last = shown; still = 0;
    while (still < 2000) begin #1000; if (shown == last) still = still + 1; else begin still = 0; last = shown; end end
    $display("cycles %0d shown %h", cycle, shown);
    $finish;
  end
  initial begin #100000000; $fatal(1, "hex-run: the simulation took too long"); end
endmodule
