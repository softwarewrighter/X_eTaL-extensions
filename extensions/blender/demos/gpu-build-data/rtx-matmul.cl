// X_eTaL-gpu: generated OpenCL C 1.2. Floored division and remainder, as X_eTaL's d_iv and m_od.
inline long xetal_idiv_long(long a, long b) { long q = a / b; return (a % b != 0 && ((a < 0) != (b < 0))) ? q - 1 : q; }
inline long xetal_imod_long(long a, long b) { return a - xetal_idiv_long(a, b) * b; }
inline int xetal_idiv_int(int a, int b) { int q = a / b; return (a % b != 0 && ((a < 0) != (b < 0))) ? q - 1 : q; }
inline int xetal_imod_int(int a, int b) { return a - xetal_idiv_int(a, b) * b; }

// matmul_tiled16_long: a by b in 16 by 16 tiles: each work-group computes a 16 by 16
// block of the result, staging a block of a and of b in local memory per
// step; the steps go from the last block of the axis to the first and
// each block's products from the last to the first, so every sum is
// folded from the right, as X_eTaL's '+ '* i_nner folds it.
__kernel void matmul_tiled16_long(__global const long* a, __global const long* b, __global long* c, const long m, const long n, const long p) {
    __local long sa[16][16];
    __local long sb[16][16];
    long lid = get_local_id(0);
    long ty = lid / 16, tx = lid % 16;
    long blocksp = (p + 16 - 1) / 16;
    long g = get_group_id(0);
    long i = (g / blocksp) * 16 + ty, j = (g % blocksp) * 16 + tx;
    long acc = 0;
    int started = 0;
    for (long s = (n + 16 - 1) / 16 - 1; s >= 0; s--) {
        long ka = s * 16 + tx, kb = s * 16 + ty;
        sa[ty][tx] = (i < m && ka < n) ? a[i * n + ka] : 0;
        sb[ty][tx] = (kb < n && j < p) ? b[kb * p + j] : 0;
        barrier(CLK_LOCAL_MEM_FENCE);
        for (long kk = 16 - 1; kk >= 0; kk--) {
            if (s * 16 + kk < n) {
                long prod = sa[ty][kk] * sb[kk][tx];
                acc = started ? prod + acc : prod;
                started = 1;
            }
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    if (i < m && j < p) c[i * p + j] = acc;
}

// k0: 12 elements, one work-item each; computes %div
__kernel void k0(__global const float* b_l, __global float* b_div) {
    size_t i = get_global_id(0);
    if (i >= 12) return;
    float t_div = (b_l[i] / 4.0f);
    b_div[i] = t_div;
}

// k1: 12 elements, one work-item each; computes %div_2
__kernel void k1(__global const float* b_l_2, __global float* b_div_2) {
    size_t i = get_global_id(0);
    if (i >= 12) return;
    float t_div_2 = (b_l_2[i] / 8.0f);
    b_div_2[i] = t_div_2;
}

// matmul_tiled16_float: a by b in 16 by 16 tiles: each work-group computes a 16 by 16
// block of the result, staging a block of a and of b in local memory per
// step; the steps go from the last block of the axis to the first and
// each block's products from the last to the first, so every sum is
// folded from the right, as X_eTaL's '+ '* i_nner folds it.
__kernel void matmul_tiled16_float(__global const float* a, __global const float* b, __global float* c, const long m, const long n, const long p) {
    __local float sa[16][16];
    __local float sb[16][16];
    long lid = get_local_id(0);
    long ty = lid / 16, tx = lid % 16;
    long blocksp = (p + 16 - 1) / 16;
    long g = get_group_id(0);
    long i = (g / blocksp) * 16 + ty, j = (g % blocksp) * 16 + tx;
    float acc = 0;
    int started = 0;
    for (long s = (n + 16 - 1) / 16 - 1; s >= 0; s--) {
        long ka = s * 16 + tx, kb = s * 16 + ty;
        sa[ty][tx] = (i < m && ka < n) ? a[i * n + ka] : 0;
        sb[ty][tx] = (kb < n && j < p) ? b[kb * p + j] : 0;
        barrier(CLK_LOCAL_MEM_FENCE);
        for (long kk = 16 - 1; kk >= 0; kk--) {
            if (s * 16 + kk < n) {
                float prod = sa[ty][kk] * sb[kk][tx];
                acc = started ? prod + acc : prod;
                started = 1;
            }
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    if (i < m && j < p) c[i * p + j] = acc;
}
