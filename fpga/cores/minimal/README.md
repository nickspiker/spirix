# Spirix Core — Register-Machine ALU

21-op register machine in plain Verilog. 8x128-bit register file, 18-bit instructions, variable-latency execution. Composes the `fpga/cores/ops/` module library into a software-sequenced ALU core.

Target: 170 MHz on ECP5-25F (multiply_pipe bottleneck).

## Architecture

```
                    18-bit instruction
                          |
              +-----------v-----------+
              |    Instruction Decode  |
              |  opcode ra rb rd fw ew |
              +-----------+-----------+
                          |
              +-----------v-----------+
              |    Register File       |
              |    8 x (64f + 64e)     |
              |    async read ports    |
              +---+---+---+---+---+---+
                  |   |   |   |   |
          +-------+   |   |   |   +-------+
          v       v   v   v   v           v
       basic   minmax round addbit  multiply  divmodsqrt  random
       (comb)  (comb) (comb) (pipe)  (pipe)    (iter)     (iter)
          |       |     |     |        |          |          |
          +-------+-----+----+--------+----------+----------+
                          |
              +-----------v-----------+
              |      Result Mux        |
              +-----------+-----------+
                          |
                     write-back
```

All modules run in parallel on register file outputs. Only the opcode-selected result is written back.

## Instruction Word (18 bits)

```
[17:13]  opcode   5-bit operation select (21 ops, room for 32)
[12:10]  ra       source register A
[9:7]    rb       source register B
[6:4]    rd       destination register
[3:2]    frac_w   fraction width: 00=8  01=16  10=32  11=64
[1:0]    exp_w    exponent width: 00=8  01=16  10=32  11=64
```

## Operations

| Opcode | Op | Module | Latency | Description |
|--------|-----|---------|---------|-------------|
| 0 | NEG | basic | 1 clk | Negate |
| 1 | ABS | basic | 1 clk | Absolute value |
| 2 | SIGN | basic | 1 clk | Sign extraction |
| 3 | SHL | basic | 1 clk | Shift left (double) |
| 4 | SHR | basic | 1 clk | Shift right (halve) |
| 5 | MIN | minmax | 1 clk | Minimum |
| 6 | MAX | minmax | 1 clk | Maximum |
| 7 | ADD | addbit_pipe | 3 clk | Addition |
| 8 | SUB | addbit_pipe | 3 clk | Subtraction |
| 9 | AND | addbit_pipe | 3 clk | Bitwise AND |
| 10 | OR | addbit_pipe | 3 clk | Bitwise OR |
| 11 | XOR | addbit_pipe | 3 clk | Bitwise XOR |
| 12 | FLOOR | round | 1 clk | Floor (toward -inf) |
| 13 | CEIL | round | 1 clk | Ceiling (toward +inf) |
| 14 | ROUND | round | 1 clk | Round to nearest |
| 15 | FRAC | micro-op | 1\|3 clk | Fractional part: SUB(a, FLOOR(a)) |
| 16 | MUL | multiply_pipe | 3 clk | Multiplication |
| 17 | DIV | divmodsqrt | FRAC+3..5 | Division |
| 18 | SQRT | divmodsqrt | FRAC+3..5 | Square root |
| 19 | MOD | divmodsqrt | FRAC+3..5 | Modulo (Python-style floored) |
| 20 | RNG | random | 4 clk (warm) | Hardware TRNG |

## Latency Classes

- **1-cycle (combinational)**: basic, minmax, round. Result valid same cycle as exec. EXEC0 -> write -> IDLE.
- **1|3-cycle (micro-op)**: FRAC. Non-normal inputs shortcut in 1 cycle (sentinels). Normal inputs decompose to round(FLOOR) + addbit_pipe(SUB) in 3 cycles, with WB clamp for precision overflow.
- **3-cycle (pipelined)**: addbit_pipe, multiply_pipe. CE-gated 2-stage pipeline. EXEC0 -> EXEC1 -> WB -> IDLE. CE asserted from S_IDLE so both S1 and S2 capture.
- **N-cycle (iterative)**: divmodsqrt, random. Start pulse in EXEC0, wait for done. EXEC0 -> WAIT... -> WB -> IDLE.

## Interface

```verilog
spirix_core #(.MAX_FRAC(64), .MAX_EXP(64)) core (
    .clk(clk), .rst(rst),

    // Instruction interface
    .instr(instr),       // 18-bit instruction word
    .exec(exec),         // pulse: execute instruction
    .busy(busy),         // high while executing
    .done(done),         // pulse: instruction complete

    // External register access (active when not busy)
    .ext_addr(addr),     // 3-bit register select (R0-R7)
    .ext_wfrac(wfrac),   // write fraction
    .ext_wexp(wexp),     // write exponent
    .ext_we(we),         // write enable
    .ext_rfrac(rfrac),   // read fraction
    .ext_rexp(rexp)      // read exponent
);
```

## Usage

1. Wait for `!busy`
2. Load operands via ext ports (`ext_we` + `ext_addr` + data)
3. Set `instr` and pulse `exec`
4. Wait for `done` pulse
5. Read result via ext ports (`ext_addr` -> `ext_rfrac`, `ext_rexp`)

External register writes are only accepted when idle and `exec` is not asserted.

## Dependencies

Requires these modules from `fpga/cores/ops/`:

| Module | File |
|--------|------|
| `spirix_neg` | `spirix_neg.v` |
| `spirix_cmp` | `spirix_cmp.v` |
| `spirix_alu_basic` | `spirix_alu_basic.v` |
| `spirix_alu_minmax` | `spirix_alu_minmax.v` |
| `spirix_alu_round` | `spirix_alu_round.v` |
| `spirix_alu_addbit_pipe` | `spirix_alu_addbit_pipe.v` |
| `spirix_alu_multiply_pipe` | `spirix_alu_multiply_pipe.v` |
| `spirix_alu_divmodsqrt` | `spirix_alu_divmodsqrt.v` |
| `spirix_alu_random` | `spirix_alu_random.v` |

No vendor primitives except `LUT4` in `spirix_alu_random` (ECP5 ring oscillators for TRNG).

## Number Format

```
value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
```

- **Fraction**: signed two's complement, N1-normalized (top two bits differ: `01...` positive, `10...` negative)
- **Exponent**: signed two's complement, unbiased
- **Sentinel**: `AMBIGUOUS_EXP = -2^(EXP_BITS-1)` encodes non-normal states (zero, infinity, exploded, vanished, undefined)

Runtime-selectable precision: 8/16/32/64-bit fraction and exponent via `frac_w` and `exp_w` instruction fields.

## Synthesis

19,441 LUT4, 16 DSP18, 13 DP16KD on ECP5-25F (Yosys+nextpnr, -nowidelut, full harness).

Core standalone (no harness): 15,820 LUT4, 16 DSP18, 96 DPR16x4.

## Comparison vs HardFloat Multi-Width

HardFloat (Berkeley) provides add, multiply, and div/sqrt — no modulus, no bitwise ops, no shifts, no min/max, no rounding ops, no RNG. It cannot do runtime-selectable width; each precision requires a separate instantiation. A comparable multi-width HardFloat unit needs 4 parallel instances of each op (binary16/32/64/128).

All numbers: ECP5, Yosys synth_ecp5, -nowidelut, 0 DSP, IEEE 754 I/O (fNToRecFN + recFNToFN converters included).

| Op | HF f16 | HF f32 | HF f64 | HF f128 | HF total (4 instances) | Spirix (1 datapath) |
|---|---|---|---|---|---|---|
| Add | 456 | 1,033 | 2,558 | 6,059 | 10,106 | ~6,700 |
| Mul | 329 | 794 | 2,067 | 6,053 | 9,243 | ~2,666 (16 DSP) |
| Div/Sqrt | 449 | 1,059 | 2,389 | 5,343 | 9,240 | 5,433 |
| **Total** | 1,234 | 2,886 | 7,014 | 17,455 | **28,589** | **~14,799** |

Spirix is ~48% smaller with a single datapath handling 16 width combinations (4 frac x 4 exp) vs HardFloat's 4 fixed IEEE widths. Spirix also includes 15 additional operations HardFloat lacks: NEG, ABS, SIGN, SHL, SHR, MIN, MAX, AND, OR, XOR, FLOOR, CEIL, ROUND, FRAC, MOD, and a hardware TRNG — all within the same 15K LUT4 budget.

HardFloat's multiply is pure-LUT (no DSP). Spirix uses 16 DSP18 for the Karatsuba multiplier. With `-nodsp`, Spirix multiply grows to ~2,131 LUT4 standalone (combinational, single-width) or larger for the multi-width pipe.

HardFloat does not support binary80 (x87 extended precision). No open-source FP library provides a multi-width IEEE core with runtime width selection.

Did I mention ours is tuned and pipelined?