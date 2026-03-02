// Top-level wrapper for Spirix F4E4 subtractor on Colorlight 5A-75B v8.0
//
// Behavior:
//   - Uses 25MHz clock to drive a slow counter
//   - Feeds counter bits into the subtractor as test inputs
//   - Blinks the onboard LED based on result_frac LSB
//   - LED is active-low (T6)

module top (
    input  wire clk,        // 25MHz clock (P6)
    output wire led         // Active-low LED (T6)
);

    // 25-bit counter: bit 24 ticks at ~0.75Hz (visible blink)
    reg [24:0] counter;
    always @(posedge clk)
        counter <= counter + 1;

    // Feed counter bits as test inputs to the subtractor
    // a = (counter[7:0], counter[8])  b = (counter[16:9], counter[17])
    wire signed [15:0] a_frac = {{8{counter[7]}}, counter[7:0]};
    wire signed [15:0] a_exp  = {{15{1'b0}}, counter[8]};
    wire signed [15:0] b_frac = {{8{counter[16]}}, counter[16:9]};
    wire signed [15:0] b_exp  = {{15{1'b0}}, counter[17]};

    wire signed [15:0] result_frac;
    wire signed [15:0] result_exp;

    spirix_subtract_f4e4 #(
        .ROUNDING_MODE(0)
    ) sub (
        .clk(1'b0),
        .rst(1'b0),
        .a_frac(a_frac),
        .a_exp(a_exp),
        .b_frac(b_frac),
        .b_exp(b_exp),
        .result_frac(result_frac),
        .result_exp(result_exp)
    );

    // Blink LED with result bit XOR'd against slow counter tick
    // Active low: 0 = on, 1 = off
    assign led = ~(result_frac[0] ^ counter[24]);

endmodule
