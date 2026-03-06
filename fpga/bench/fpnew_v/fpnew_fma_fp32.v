module lzc (
	in_i,
	cnt_o,
	empty_o
);
	reg _sv2v_0;
	parameter [31:0] WIDTH = 2;
	parameter [0:0] MODE = 1'b0;
	function automatic [31:0] cf_math_pkg_idx_width;
		input reg [31:0] num_idx;
		cf_math_pkg_idx_width = (num_idx > 32'd1 ? $unsigned($clog2(num_idx)) : 32'd1);
	endfunction
	parameter [31:0] CNT_WIDTH = cf_math_pkg_idx_width(WIDTH);
	input wire [WIDTH - 1:0] in_i;
	output wire [CNT_WIDTH - 1:0] cnt_o;
	output wire empty_o;
	generate
		if (WIDTH == 1) begin : gen_degenerate_lzc
			assign cnt_o[0] = !in_i[0];
			assign empty_o = !in_i[0];
		end
		else begin : gen_lzc
			localparam [31:0] NumLevels = $clog2(WIDTH);
			wire [(WIDTH * NumLevels) - 1:0] index_lut;
			wire [(2 ** NumLevels) - 1:0] sel_nodes;
			wire [((2 ** NumLevels) * NumLevels) - 1:0] index_nodes;
			reg [WIDTH - 1:0] in_tmp;
			always @(*) begin : flip_vector
				if (_sv2v_0)
					;
				begin : sv2v_autoblock_1
					reg [31:0] i;
					for (i = 0; i < WIDTH; i = i + 1)
						in_tmp[i] = (MODE ? in_i[(WIDTH - 1) - i] : in_i[i]);
				end
			end
			genvar _gv_j_1;
			for (_gv_j_1 = 0; $unsigned(_gv_j_1) < WIDTH; _gv_j_1 = _gv_j_1 + 1) begin : g_index_lut
				localparam j = _gv_j_1;
				function automatic [NumLevels - 1:0] sv2v_cast_677FF;
					input reg [NumLevels - 1:0] inp;
					sv2v_cast_677FF = inp;
				endfunction
				assign index_lut[j * NumLevels+:NumLevels] = sv2v_cast_677FF($unsigned(j));
			end
			genvar _gv_level_1;
			for (_gv_level_1 = 0; $unsigned(_gv_level_1) < NumLevels; _gv_level_1 = _gv_level_1 + 1) begin : g_levels
				localparam level = _gv_level_1;
				if ($unsigned(level) == (NumLevels - 1)) begin : g_last_level
					genvar _gv_k_1;
					for (_gv_k_1 = 0; _gv_k_1 < (2 ** level); _gv_k_1 = _gv_k_1 + 1) begin : g_level
						localparam k = _gv_k_1;
						if (($unsigned(k) * 2) < (WIDTH - 1)) begin : g_reduce
							assign sel_nodes[((2 ** level) - 1) + k] = in_tmp[k * 2] | in_tmp[(k * 2) + 1];
							assign index_nodes[(((2 ** level) - 1) + k) * NumLevels+:NumLevels] = (in_tmp[k * 2] == 1'b1 ? index_lut[(k * 2) * NumLevels+:NumLevels] : index_lut[((k * 2) + 1) * NumLevels+:NumLevels]);
						end
						if (($unsigned(k) * 2) == (WIDTH - 1)) begin : g_base
							assign sel_nodes[((2 ** level) - 1) + k] = in_tmp[k * 2];
							assign index_nodes[(((2 ** level) - 1) + k) * NumLevels+:NumLevels] = index_lut[(k * 2) * NumLevels+:NumLevels];
						end
						if (($unsigned(k) * 2) > (WIDTH - 1)) begin : g_out_of_range
							assign sel_nodes[((2 ** level) - 1) + k] = 1'b0;
							assign index_nodes[(((2 ** level) - 1) + k) * NumLevels+:NumLevels] = 1'sb0;
						end
					end
				end
				else begin : g_not_last_level
					genvar _gv_l_1;
					for (_gv_l_1 = 0; _gv_l_1 < (2 ** level); _gv_l_1 = _gv_l_1 + 1) begin : g_level
						localparam l = _gv_l_1;
						assign sel_nodes[((2 ** level) - 1) + l] = sel_nodes[((2 ** (level + 1)) - 1) + (l * 2)] | sel_nodes[(((2 ** (level + 1)) - 1) + (l * 2)) + 1];
						assign index_nodes[(((2 ** level) - 1) + l) * NumLevels+:NumLevels] = (sel_nodes[((2 ** (level + 1)) - 1) + (l * 2)] == 1'b1 ? index_nodes[(((2 ** (level + 1)) - 1) + (l * 2)) * NumLevels+:NumLevels] : index_nodes[((((2 ** (level + 1)) - 1) + (l * 2)) + 1) * NumLevels+:NumLevels]);
					end
				end
			end
			assign cnt_o = (NumLevels > $unsigned(0) ? index_nodes[0+:NumLevels] : {$clog2(WIDTH) {1'b0}});
			assign empty_o = (NumLevels > $unsigned(0) ? ~sel_nodes[0] : ~(|in_i));
		end
	endgenerate
	initial _sv2v_0 = 0;
endmodule
module fpnew_classifier (
	operands_i,
	is_boxed_i,
	info_o
);
	reg _sv2v_0;
	localparam [31:0] fpnew_pkg_NUM_FP_FORMATS = 5;
	localparam [31:0] fpnew_pkg_FP_FORMAT_BITS = 3;
	function automatic [2:0] sv2v_cast_5D882;
		input reg [2:0] inp;
		sv2v_cast_5D882 = inp;
	endfunction
	parameter [2:0] FpFormat = sv2v_cast_5D882(0);
	parameter [31:0] NumOperands = 1;
	localparam [319:0] fpnew_pkg_FP_ENCODINGS = 320'h8000000170000000b00000034000000050000000a00000005000000020000000800000007;
	function automatic [31:0] fpnew_pkg_fp_width;
		input reg [2:0] fmt;
		fpnew_pkg_fp_width = (fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 63-:32] + fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 31-:32]) + 1;
	endfunction
	localparam [31:0] WIDTH = fpnew_pkg_fp_width(FpFormat);
	input wire [(NumOperands * WIDTH) - 1:0] operands_i;
	input wire [NumOperands - 1:0] is_boxed_i;
	output reg [(NumOperands * 8) - 1:0] info_o;
	function automatic [31:0] fpnew_pkg_exp_bits;
		input reg [2:0] fmt;
		fpnew_pkg_exp_bits = fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 63-:32];
	endfunction
	localparam [31:0] EXP_BITS = fpnew_pkg_exp_bits(FpFormat);
	function automatic [31:0] fpnew_pkg_man_bits;
		input reg [2:0] fmt;
		fpnew_pkg_man_bits = fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 31-:32];
	endfunction
	localparam [31:0] MAN_BITS = fpnew_pkg_man_bits(FpFormat);
	genvar _gv_op_1;
	function automatic signed [31:0] sv2v_cast_32_signed;
		input reg signed [31:0] inp;
		sv2v_cast_32_signed = inp;
	endfunction
	generate
		for (_gv_op_1 = 0; _gv_op_1 < sv2v_cast_32_signed(NumOperands); _gv_op_1 = _gv_op_1 + 1) begin : gen_num_values
			localparam op = _gv_op_1;
			reg [((1 + EXP_BITS) + MAN_BITS) - 1:0] value;
			reg is_boxed;
			reg is_normal;
			reg is_inf;
			reg is_nan;
			reg is_signalling;
			reg is_quiet;
			reg is_zero;
			reg is_subnormal;
			always @(*) begin : classify_input
				if (_sv2v_0)
					;
				value = operands_i[op * WIDTH+:WIDTH];
				is_boxed = is_boxed_i[op];
				is_normal = (is_boxed && (value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] != {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb0}})) && (value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] != {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb1}});
				is_zero = (is_boxed && (value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] == {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb0}})) && (value[MAN_BITS - 1-:MAN_BITS] == {MAN_BITS * 1 {1'sb0}});
				is_subnormal = (is_boxed && (value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] == {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb0}})) && !is_zero;
				is_inf = is_boxed && ((value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] == {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb1}}) && (value[MAN_BITS - 1-:MAN_BITS] == {MAN_BITS * 1 {1'sb0}}));
				is_nan = !is_boxed || ((value[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)] == {((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1) * 1 {1'sb1}}) && (value[MAN_BITS - 1-:MAN_BITS] != {MAN_BITS * 1 {1'sb0}}));
				is_signalling = (is_boxed && is_nan) && (value[(MAN_BITS - 1) - ((MAN_BITS - 1) - (MAN_BITS - 1))] == 1'b0);
				is_quiet = is_nan && !is_signalling;
				info_o[(op * 8) + 7] = is_normal;
				info_o[(op * 8) + 6] = is_subnormal;
				info_o[(op * 8) + 5] = is_zero;
				info_o[(op * 8) + 4] = is_inf;
				info_o[(op * 8) + 3] = is_nan;
				info_o[(op * 8) + 2] = is_signalling;
				info_o[(op * 8) + 1] = is_quiet;
				info_o[op * 8] = is_boxed;
			end
		end
	endgenerate
	initial _sv2v_0 = 0;
endmodule
module fpnew_rounding (
	abs_value_i,
	sign_i,
	round_sticky_bits_i,
	rnd_mode_i,
	effective_subtraction_i,
	abs_rounded_o,
	sign_o,
	exact_zero_o
);
	reg _sv2v_0;
	parameter [31:0] AbsWidth = 2;
	input wire [AbsWidth - 1:0] abs_value_i;
	input wire sign_i;
	input wire [1:0] round_sticky_bits_i;
	input wire [2:0] rnd_mode_i;
	input wire effective_subtraction_i;
	output wire [AbsWidth - 1:0] abs_rounded_o;
	output wire sign_o;
	output wire exact_zero_o;
	reg round_up;
	localparam [0:0] fpnew_pkg_DONT_CARE = 1'b1;
	always @(*) begin : rounding_decision
		if (_sv2v_0)
			;
		(* full_case, parallel_case *)
		case (rnd_mode_i)
			3'b000:
				(* full_case, parallel_case *)
				case (round_sticky_bits_i)
					2'b00, 2'b01: round_up = 1'b0;
					2'b10: round_up = abs_value_i[0];
					2'b11: round_up = 1'b1;
					default: round_up = fpnew_pkg_DONT_CARE;
				endcase
			3'b001: round_up = 1'b0;
			3'b010: round_up = (|round_sticky_bits_i ? sign_i : 1'b0);
			3'b011: round_up = (|round_sticky_bits_i ? ~sign_i : 1'b0);
			3'b100: round_up = round_sticky_bits_i[1];
			3'b101: round_up = ~abs_value_i[0] & |round_sticky_bits_i;
			default: round_up = fpnew_pkg_DONT_CARE;
		endcase
	end
	assign abs_rounded_o = abs_value_i + round_up;
	assign exact_zero_o = (abs_value_i == {AbsWidth {1'sb0}}) && (round_sticky_bits_i == {2 {1'sb0}});
	assign sign_o = (exact_zero_o && effective_subtraction_i ? rnd_mode_i == 3'b010 : sign_i);
	initial _sv2v_0 = 0;
endmodule
module fpnew_fma (
	clk_i,
	rst_ni,
	operands_i,
	is_boxed_i,
	rnd_mode_i,
	op_i,
	op_mod_i,
	tag_i,
	mask_i,
	aux_i,
	in_valid_i,
	in_ready_o,
	flush_i,
	result_o,
	status_o,
	extension_bit_o,
	tag_o,
	mask_o,
	aux_o,
	out_valid_o,
	out_ready_i,
	busy_o,
	reg_ena_i,
	early_out_valid_o
);
	reg _sv2v_0;
	localparam [31:0] fpnew_pkg_NUM_FP_FORMATS = 5;
	localparam [31:0] fpnew_pkg_FP_FORMAT_BITS = 3;
	function automatic [2:0] sv2v_cast_5D882;
		input reg [2:0] inp;
		sv2v_cast_5D882 = inp;
	endfunction
	parameter [2:0] FpFormat = sv2v_cast_5D882(0);
	parameter [31:0] NumPipeRegs = 0;
	parameter [1:0] PipeConfig = 2'd0;
	localparam [319:0] fpnew_pkg_FP_ENCODINGS = 320'h8000000170000000b00000034000000050000000a00000005000000020000000800000007;
	function automatic [31:0] fpnew_pkg_fp_width;
		input reg [2:0] fmt;
		fpnew_pkg_fp_width = (fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 63-:32] + fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 31-:32]) + 1;
	endfunction
	localparam [31:0] WIDTH = fpnew_pkg_fp_width(FpFormat);
	localparam [31:0] ExtRegEnaWidth = (NumPipeRegs == 0 ? 1 : NumPipeRegs);
	input wire clk_i;
	input wire rst_ni;
	input wire [(3 * WIDTH) - 1:0] operands_i;
	input wire [2:0] is_boxed_i;
	input wire [2:0] rnd_mode_i;
	localparam [31:0] fpnew_pkg_OP_BITS = 4;
	input wire [3:0] op_i;
	input wire op_mod_i;
	input wire tag_i;
	input wire mask_i;
	input wire aux_i;
	input wire in_valid_i;
	output wire in_ready_o;
	input wire flush_i;
	output wire [WIDTH - 1:0] result_o;
	output wire [4:0] status_o;
	output wire extension_bit_o;
	output wire tag_o;
	output wire mask_o;
	output wire aux_o;
	output wire out_valid_o;
	input wire out_ready_i;
	output wire busy_o;
	input wire [ExtRegEnaWidth - 1:0] reg_ena_i;
	output wire early_out_valid_o;
	function automatic [31:0] fpnew_pkg_exp_bits;
		input reg [2:0] fmt;
		fpnew_pkg_exp_bits = fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 63-:32];
	endfunction
	localparam [31:0] EXP_BITS = fpnew_pkg_exp_bits(FpFormat);
	function automatic [31:0] fpnew_pkg_man_bits;
		input reg [2:0] fmt;
		fpnew_pkg_man_bits = fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 31-:32];
	endfunction
	localparam [31:0] MAN_BITS = fpnew_pkg_man_bits(FpFormat);
	function automatic [31:0] fpnew_pkg_bias;
		input reg [2:0] fmt;
		fpnew_pkg_bias = $unsigned((2 ** (fpnew_pkg_FP_ENCODINGS[((4 - fmt) * 64) + 63-:32] - 1)) - 1);
	endfunction
	localparam [31:0] BIAS = fpnew_pkg_bias(FpFormat);
	localparam [31:0] PRECISION_BITS = MAN_BITS + 1;
	localparam [31:0] LOWER_SUM_WIDTH = (2 * PRECISION_BITS) + 3;
	localparam [31:0] LZC_RESULT_WIDTH = $clog2(LOWER_SUM_WIDTH);
	function automatic signed [31:0] fpnew_pkg_maximum;
		input reg signed [31:0] a;
		input reg signed [31:0] b;
		fpnew_pkg_maximum = (a > b ? a : b);
	endfunction
	localparam [31:0] EXP_WIDTH = $unsigned(fpnew_pkg_maximum(EXP_BITS + 2, LZC_RESULT_WIDTH));
	localparam [31:0] SHIFT_AMOUNT_WIDTH = $clog2((3 * PRECISION_BITS) + 5);
	localparam NUM_INP_REGS = (PipeConfig == 2'd0 ? NumPipeRegs : (PipeConfig == 2'd3 ? (NumPipeRegs + 1) / 3 : 0));
	localparam NUM_MID_REGS = (PipeConfig == 2'd2 ? NumPipeRegs : (PipeConfig == 2'd3 ? (NumPipeRegs + 2) / 3 : 0));
	localparam NUM_OUT_REGS = (PipeConfig == 2'd1 ? NumPipeRegs : (PipeConfig == 2'd3 ? NumPipeRegs / 3 : 0));
	reg [((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) - (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0)) + 1) * WIDTH) + (((0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) * WIDTH) - 1) : ((((0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)) + 1) * WIDTH) + (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) * WIDTH) - 1)):((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) * WIDTH : (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) * WIDTH)] inp_pipe_operands_q;
	reg [(0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1):(0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0)] inp_pipe_is_boxed_q;
	reg [(0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1):(0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0)] inp_pipe_rnd_mode_q;
	reg [(0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * fpnew_pkg_OP_BITS) + ((NUM_INP_REGS * fpnew_pkg_OP_BITS) - 1) : ((NUM_INP_REGS + 1) * fpnew_pkg_OP_BITS) - 1):(0 >= NUM_INP_REGS ? NUM_INP_REGS * fpnew_pkg_OP_BITS : 0)] inp_pipe_op_q;
	reg [0:NUM_INP_REGS] inp_pipe_op_mod_q;
	reg [0:NUM_INP_REGS] inp_pipe_tag_q;
	reg [0:NUM_INP_REGS] inp_pipe_mask_q;
	reg [0:NUM_INP_REGS] inp_pipe_aux_q;
	reg [0:NUM_INP_REGS] inp_pipe_valid_q;
	wire [0:NUM_INP_REGS] inp_pipe_ready;
	wire [3 * WIDTH:1] sv2v_tmp_A0212;
	assign sv2v_tmp_A0212 = operands_i;
	always @(*) inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3 : ((0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3 : ((0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3] = sv2v_tmp_A0212;
	wire [3:1] sv2v_tmp_7BE72;
	assign sv2v_tmp_7BE72 = is_boxed_i;
	always @(*) inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3+:3] = sv2v_tmp_7BE72;
	wire [3:1] sv2v_tmp_FB312;
	assign sv2v_tmp_FB312 = rnd_mode_i;
	always @(*) inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * 3+:3] = sv2v_tmp_FB312;
	wire [4:1] sv2v_tmp_9DB1A;
	assign sv2v_tmp_9DB1A = op_i;
	always @(*) inp_pipe_op_q[(0 >= NUM_INP_REGS ? 0 : NUM_INP_REGS) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS] = sv2v_tmp_9DB1A;
	wire [1:1] sv2v_tmp_72E02;
	assign sv2v_tmp_72E02 = op_mod_i;
	always @(*) inp_pipe_op_mod_q[0] = sv2v_tmp_72E02;
	wire [1:1] sv2v_tmp_DE624;
	assign sv2v_tmp_DE624 = tag_i;
	always @(*) inp_pipe_tag_q[0] = sv2v_tmp_DE624;
	wire [1:1] sv2v_tmp_AE6A6;
	assign sv2v_tmp_AE6A6 = mask_i;
	always @(*) inp_pipe_mask_q[0] = sv2v_tmp_AE6A6;
	wire [1:1] sv2v_tmp_683C4;
	assign sv2v_tmp_683C4 = aux_i;
	always @(*) inp_pipe_aux_q[0] = sv2v_tmp_683C4;
	wire [1:1] sv2v_tmp_CFC25;
	assign sv2v_tmp_CFC25 = in_valid_i;
	always @(*) inp_pipe_valid_q[0] = sv2v_tmp_CFC25;
	assign in_ready_o = inp_pipe_ready[0];
	genvar _gv_i_1;
	function automatic [3:0] sv2v_cast_4CD2E;
		input reg [3:0] inp;
		sv2v_cast_4CD2E = inp;
	endfunction
	generate
		for (_gv_i_1 = 0; _gv_i_1 < NUM_INP_REGS; _gv_i_1 = _gv_i_1 + 1) begin : gen_input_pipeline
			localparam i = _gv_i_1;
			wire reg_ena;
			assign inp_pipe_ready[i] = inp_pipe_ready[i + 1] | ~inp_pipe_valid_q[i + 1];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_valid_q[i + 1] <= 1'b0;
				else
					inp_pipe_valid_q[i + 1] <= (flush_i ? 1'b0 : (inp_pipe_ready[i] ? inp_pipe_valid_q[i] : inp_pipe_valid_q[i + 1]));
			assign reg_ena = (inp_pipe_ready[i] & inp_pipe_valid_q[i]) | reg_ena_i[i];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3] <= 1'sb0;
				else
					inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3] <= (reg_ena ? inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3 : ((0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3 : ((0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3] : inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3 : ((0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3] <= 1'sb0;
				else
					inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3] <= (reg_ena ? inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3+:3] : inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3] <= 3'b000;
				else
					inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3] <= (reg_ena ? inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * 3+:3] : inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * 3+:3]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_op_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS] <= sv2v_cast_4CD2E(0);
				else
					inp_pipe_op_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS] <= (reg_ena ? inp_pipe_op_q[(0 >= NUM_INP_REGS ? i : NUM_INP_REGS - i) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS] : inp_pipe_op_q[(0 >= NUM_INP_REGS ? i + 1 : NUM_INP_REGS - (i + 1)) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_op_mod_q[i + 1] <= 1'sb0;
				else
					inp_pipe_op_mod_q[i + 1] <= (reg_ena ? inp_pipe_op_mod_q[i] : inp_pipe_op_mod_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_tag_q[i + 1] <= 1'b0;
				else
					inp_pipe_tag_q[i + 1] <= (reg_ena ? inp_pipe_tag_q[i] : inp_pipe_tag_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_mask_q[i + 1] <= 1'sb0;
				else
					inp_pipe_mask_q[i + 1] <= (reg_ena ? inp_pipe_mask_q[i] : inp_pipe_mask_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					inp_pipe_aux_q[i + 1] <= 1'b0;
				else
					inp_pipe_aux_q[i + 1] <= (reg_ena ? inp_pipe_aux_q[i] : inp_pipe_aux_q[i + 1]);
		end
	endgenerate
	wire [23:0] info_q;
	fpnew_classifier #(
		.FpFormat(FpFormat),
		.NumOperands(3)
	) i_class_inputs(
		.operands_i(inp_pipe_operands_q[WIDTH * ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3 : ((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 2) : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3 : ((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1)))+:WIDTH * 3]),
		.is_boxed_i(inp_pipe_is_boxed_q[(0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3+:3]),
		.info_o(info_q)
	);
	reg [((1 + EXP_BITS) + MAN_BITS) - 1:0] operand_a;
	reg [((1 + EXP_BITS) + MAN_BITS) - 1:0] operand_b;
	reg [((1 + EXP_BITS) + MAN_BITS) - 1:0] operand_c;
	reg [7:0] info_a;
	reg [7:0] info_b;
	reg [7:0] info_c;
	localparam [0:0] fpnew_pkg_DONT_CARE = 1'b1;
	function automatic [EXP_BITS - 1:0] sv2v_cast_51E93;
		input reg [EXP_BITS - 1:0] inp;
		sv2v_cast_51E93 = inp;
	endfunction
	function automatic [MAN_BITS - 1:0] sv2v_cast_DBAB7;
		input reg [MAN_BITS - 1:0] inp;
		sv2v_cast_DBAB7 = inp;
	endfunction
	function automatic [EXP_BITS - 1:0] sv2v_cast_98E28;
		input reg [EXP_BITS - 1:0] inp;
		sv2v_cast_98E28 = inp;
	endfunction
	function automatic [MAN_BITS - 1:0] sv2v_cast_D5F4C;
		input reg [MAN_BITS - 1:0] inp;
		sv2v_cast_D5F4C = inp;
	endfunction
	always @(*) begin : op_select
		if (_sv2v_0)
			;
		operand_a = inp_pipe_operands_q[((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? (0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3 : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - (((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1))) * WIDTH+:WIDTH];
		operand_b = inp_pipe_operands_q[((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 1 : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - ((((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 1) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1))) * WIDTH+:WIDTH];
		operand_c = inp_pipe_operands_q[((0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1) >= (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) ? ((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 2 : (0 >= NUM_INP_REGS ? NUM_INP_REGS * 3 : 0) - ((((0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3) + 2) - (0 >= NUM_INP_REGS ? ((1 - NUM_INP_REGS) * 3) + ((NUM_INP_REGS * 3) - 1) : ((NUM_INP_REGS + 1) * 3) - 1))) * WIDTH+:WIDTH];
		info_a = info_q[0+:8];
		info_b = info_q[8+:8];
		info_c = info_q[16+:8];
		operand_c[1 + (EXP_BITS + (MAN_BITS - 1))] = operand_c[1 + (EXP_BITS + (MAN_BITS - 1))] ^ inp_pipe_op_mod_q[NUM_INP_REGS];
		(* full_case, parallel_case *)
		case (inp_pipe_op_q[(0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * fpnew_pkg_OP_BITS+:fpnew_pkg_OP_BITS])
			sv2v_cast_4CD2E(0):
				;
			sv2v_cast_4CD2E(1): operand_a[1 + (EXP_BITS + (MAN_BITS - 1))] = ~operand_a[1 + (EXP_BITS + (MAN_BITS - 1))];
			sv2v_cast_4CD2E(2), sv2v_cast_4CD2E(15): begin
				operand_a = {1'b0, sv2v_cast_51E93(BIAS), sv2v_cast_DBAB7(1'sb0)};
				info_a = 8'b10000001;
			end
			sv2v_cast_4CD2E(3): begin
				if (inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3+:3] == 3'b010)
					operand_c = {1'b0, sv2v_cast_98E28(1'sb0), sv2v_cast_DBAB7(1'sb0)};
				else
					operand_c = {1'b1, sv2v_cast_98E28(1'sb0), sv2v_cast_DBAB7(1'sb0)};
				info_c = 8'b00100001;
			end
			default: begin
				operand_a = {fpnew_pkg_DONT_CARE, sv2v_cast_51E93(fpnew_pkg_DONT_CARE), sv2v_cast_D5F4C(fpnew_pkg_DONT_CARE)};
				operand_b = {fpnew_pkg_DONT_CARE, sv2v_cast_51E93(fpnew_pkg_DONT_CARE), sv2v_cast_D5F4C(fpnew_pkg_DONT_CARE)};
				operand_c = {fpnew_pkg_DONT_CARE, sv2v_cast_51E93(fpnew_pkg_DONT_CARE), sv2v_cast_D5F4C(fpnew_pkg_DONT_CARE)};
				info_a = {fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE};
				info_b = {fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE};
				info_c = {fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE, fpnew_pkg_DONT_CARE};
			end
		endcase
	end
	wire any_operand_inf;
	wire any_operand_nan;
	wire signalling_nan;
	wire effective_subtraction;
	wire tentative_sign;
	assign any_operand_inf = |{info_a[4], info_b[4], info_c[4]};
	assign any_operand_nan = |{info_a[3], info_b[3], info_c[3]};
	assign signalling_nan = |{info_a[2], info_b[2], info_c[2]};
	assign effective_subtraction = (operand_a[1 + (EXP_BITS + (MAN_BITS - 1))] ^ operand_b[1 + (EXP_BITS + (MAN_BITS - 1))]) ^ operand_c[1 + (EXP_BITS + (MAN_BITS - 1))];
	assign tentative_sign = operand_a[1 + (EXP_BITS + (MAN_BITS - 1))] ^ operand_b[1 + (EXP_BITS + (MAN_BITS - 1))];
	reg [((1 + EXP_BITS) + MAN_BITS) - 1:0] special_result;
	reg [4:0] special_status;
	reg result_is_special;
	always @(*) begin : special_cases
		if (_sv2v_0)
			;
		special_result = {1'b0, sv2v_cast_98E28(1'sb1), sv2v_cast_D5F4C(2 ** (MAN_BITS - 1))};
		special_status = 1'sb0;
		result_is_special = 1'b0;
		if ((info_a[4] && info_b[5]) || (info_a[5] && info_b[4])) begin
			result_is_special = 1'b1;
			special_status[4] = 1'b1;
		end
		else if (any_operand_nan) begin
			result_is_special = 1'b1;
			special_status[4] = signalling_nan;
		end
		else if (any_operand_inf) begin
			result_is_special = 1'b1;
			if (((info_a[4] || info_b[4]) && info_c[4]) && effective_subtraction)
				special_status[4] = 1'b1;
			else if (info_a[4] || info_b[4])
				special_result = {operand_a[1 + (EXP_BITS + (MAN_BITS - 1))] ^ operand_b[1 + (EXP_BITS + (MAN_BITS - 1))], sv2v_cast_98E28(1'sb1), sv2v_cast_DBAB7(1'sb0)};
			else if (info_c[4])
				special_result = {operand_c[1 + (EXP_BITS + (MAN_BITS - 1))], sv2v_cast_98E28(1'sb1), sv2v_cast_DBAB7(1'sb0)};
		end
	end
	wire signed [EXP_WIDTH - 1:0] exponent_a;
	wire signed [EXP_WIDTH - 1:0] exponent_b;
	wire signed [EXP_WIDTH - 1:0] exponent_c;
	wire signed [EXP_WIDTH - 1:0] exponent_addend;
	wire signed [EXP_WIDTH - 1:0] exponent_product;
	wire signed [EXP_WIDTH - 1:0] exponent_difference;
	wire signed [EXP_WIDTH - 1:0] tentative_exponent;
	assign exponent_a = $signed({1'b0, operand_a[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)]});
	assign exponent_b = $signed({1'b0, operand_b[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)]});
	assign exponent_c = $signed({1'b0, operand_c[EXP_BITS + (MAN_BITS - 1)-:((EXP_BITS + (MAN_BITS - 1)) >= (MAN_BITS + 0) ? ((EXP_BITS + (MAN_BITS - 1)) - (MAN_BITS + 0)) + 1 : ((MAN_BITS + 0) - (EXP_BITS + (MAN_BITS - 1))) + 1)]});
	assign exponent_addend = $signed(exponent_c + $signed({1'b0, ~info_c[7]}));
	assign exponent_product = (info_a[5] || info_b[5] ? 2 - $signed(BIAS) : $signed((((exponent_a + info_a[6]) + exponent_b) + info_b[6]) - $signed(BIAS)));
	assign exponent_difference = exponent_addend - exponent_product;
	assign tentative_exponent = (exponent_difference > 0 ? exponent_addend : exponent_product);
	reg [SHIFT_AMOUNT_WIDTH - 1:0] addend_shamt;
	always @(*) begin : addend_shift_amount
		if (_sv2v_0)
			;
		if (exponent_difference <= $signed((-2 * PRECISION_BITS) - 1))
			addend_shamt = (3 * PRECISION_BITS) + 4;
		else if (exponent_difference <= $signed(PRECISION_BITS + 2))
			addend_shamt = $unsigned(($signed(PRECISION_BITS) + 3) - exponent_difference);
		else
			addend_shamt = 0;
	end
	wire [PRECISION_BITS - 1:0] mantissa_a;
	wire [PRECISION_BITS - 1:0] mantissa_b;
	wire [PRECISION_BITS - 1:0] mantissa_c;
	wire [(2 * PRECISION_BITS) - 1:0] product;
	wire [(3 * PRECISION_BITS) + 3:0] product_shifted;
	assign mantissa_a = {info_a[7], operand_a[MAN_BITS - 1-:MAN_BITS]};
	assign mantissa_b = {info_b[7], operand_b[MAN_BITS - 1-:MAN_BITS]};
	assign mantissa_c = {info_c[7], operand_c[MAN_BITS - 1-:MAN_BITS]};
	assign product = mantissa_a * mantissa_b;
	assign product_shifted = product << 2;
	wire [(3 * PRECISION_BITS) + 3:0] addend_after_shift;
	wire [PRECISION_BITS - 1:0] addend_sticky_bits;
	wire sticky_before_add;
	wire [(3 * PRECISION_BITS) + 3:0] addend_shifted;
	wire inject_carry_in;
	assign {addend_after_shift, addend_sticky_bits} = (mantissa_c << ((3 * PRECISION_BITS) + 4)) >> addend_shamt;
	assign sticky_before_add = |addend_sticky_bits;
	assign addend_shifted = (effective_subtraction ? ~addend_after_shift : addend_after_shift);
	assign inject_carry_in = effective_subtraction & ~sticky_before_add;
	wire [(3 * PRECISION_BITS) + 4:0] sum_pos;
	wire [(3 * PRECISION_BITS) + 4:0] sum_neg;
	wire sum_carry;
	wire [(3 * PRECISION_BITS) + 3:0] sum;
	wire final_sign;
	assign sum_pos = (product_shifted + addend_shifted) + inject_carry_in;
	assign sum_carry = sum_pos[(3 * PRECISION_BITS) + 4];
	assign sum_neg = addend_after_shift - product_shifted;
	assign sum = (effective_subtraction && ~sum_carry ? sum_neg : sum_pos);
	assign final_sign = (effective_subtraction && (sum_carry == tentative_sign) ? 1'b1 : (effective_subtraction ? 1'b0 : tentative_sign));
	wire effective_subtraction_q;
	wire signed [EXP_WIDTH - 1:0] exponent_product_q;
	wire signed [EXP_WIDTH - 1:0] exponent_difference_q;
	wire signed [EXP_WIDTH - 1:0] tentative_exponent_q;
	wire [SHIFT_AMOUNT_WIDTH - 1:0] addend_shamt_q;
	wire sticky_before_add_q;
	wire [(3 * PRECISION_BITS) + 3:0] sum_q;
	wire final_sign_q;
	wire [2:0] rnd_mode_q;
	wire result_is_special_q;
	wire [((1 + EXP_BITS) + MAN_BITS) - 1:0] special_result_q;
	wire [4:0] special_status_q;
	reg [0:NUM_MID_REGS] mid_pipe_eff_sub_q;
	reg signed [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * EXP_WIDTH) + ((NUM_MID_REGS * EXP_WIDTH) - 1) : ((NUM_MID_REGS + 1) * EXP_WIDTH) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * EXP_WIDTH : 0)] mid_pipe_exp_prod_q;
	reg signed [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * EXP_WIDTH) + ((NUM_MID_REGS * EXP_WIDTH) - 1) : ((NUM_MID_REGS + 1) * EXP_WIDTH) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * EXP_WIDTH : 0)] mid_pipe_exp_diff_q;
	reg signed [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * EXP_WIDTH) + ((NUM_MID_REGS * EXP_WIDTH) - 1) : ((NUM_MID_REGS + 1) * EXP_WIDTH) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * EXP_WIDTH : 0)] mid_pipe_tent_exp_q;
	reg [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * SHIFT_AMOUNT_WIDTH) + ((NUM_MID_REGS * SHIFT_AMOUNT_WIDTH) - 1) : ((NUM_MID_REGS + 1) * SHIFT_AMOUNT_WIDTH) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * SHIFT_AMOUNT_WIDTH : 0)] mid_pipe_add_shamt_q;
	reg [0:NUM_MID_REGS] mid_pipe_sticky_q;
	reg [(0 >= NUM_MID_REGS ? (((3 * PRECISION_BITS) + 3) >= 0 ? ((1 - NUM_MID_REGS) * ((3 * PRECISION_BITS) + 4)) + ((NUM_MID_REGS * ((3 * PRECISION_BITS) + 4)) - 1) : ((1 - NUM_MID_REGS) * (1 - ((3 * PRECISION_BITS) + 3))) + ((((3 * PRECISION_BITS) + 3) + (NUM_MID_REGS * (1 - ((3 * PRECISION_BITS) + 3)))) - 1)) : (((3 * PRECISION_BITS) + 3) >= 0 ? ((NUM_MID_REGS + 1) * ((3 * PRECISION_BITS) + 4)) - 1 : ((NUM_MID_REGS + 1) * (1 - ((3 * PRECISION_BITS) + 3))) + ((3 * PRECISION_BITS) + 2))):(0 >= NUM_MID_REGS ? (((3 * PRECISION_BITS) + 3) >= 0 ? NUM_MID_REGS * ((3 * PRECISION_BITS) + 4) : ((3 * PRECISION_BITS) + 3) + (NUM_MID_REGS * (1 - ((3 * PRECISION_BITS) + 3)))) : (((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3))] mid_pipe_sum_q;
	reg [0:NUM_MID_REGS] mid_pipe_final_sign_q;
	reg [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * 3) + ((NUM_MID_REGS * 3) - 1) : ((NUM_MID_REGS + 1) * 3) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * 3 : 0)] mid_pipe_rnd_mode_q;
	reg [0:NUM_MID_REGS] mid_pipe_res_is_spec_q;
	reg [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * ((1 + EXP_BITS) + MAN_BITS)) + ((NUM_MID_REGS * ((1 + EXP_BITS) + MAN_BITS)) - 1) : ((NUM_MID_REGS + 1) * ((1 + EXP_BITS) + MAN_BITS)) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * ((1 + EXP_BITS) + MAN_BITS) : 0)] mid_pipe_spec_res_q;
	reg [(0 >= NUM_MID_REGS ? ((1 - NUM_MID_REGS) * 5) + ((NUM_MID_REGS * 5) - 1) : ((NUM_MID_REGS + 1) * 5) - 1):(0 >= NUM_MID_REGS ? NUM_MID_REGS * 5 : 0)] mid_pipe_spec_stat_q;
	reg [0:NUM_MID_REGS] mid_pipe_tag_q;
	reg [0:NUM_MID_REGS] mid_pipe_mask_q;
	reg [0:NUM_MID_REGS] mid_pipe_aux_q;
	reg [0:NUM_MID_REGS] mid_pipe_valid_q;
	wire [0:NUM_MID_REGS] mid_pipe_ready;
	wire [1:1] sv2v_tmp_301F1;
	assign sv2v_tmp_301F1 = effective_subtraction;
	always @(*) mid_pipe_eff_sub_q[0] = sv2v_tmp_301F1;
	wire [EXP_WIDTH * 1:1] sv2v_tmp_6C4A1;
	assign sv2v_tmp_6C4A1 = exponent_product;
	always @(*) mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH] = sv2v_tmp_6C4A1;
	wire [EXP_WIDTH * 1:1] sv2v_tmp_961F4;
	assign sv2v_tmp_961F4 = exponent_difference;
	always @(*) mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH] = sv2v_tmp_961F4;
	wire [EXP_WIDTH * 1:1] sv2v_tmp_1E48E;
	assign sv2v_tmp_1E48E = tentative_exponent;
	always @(*) mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH] = sv2v_tmp_1E48E;
	wire [SHIFT_AMOUNT_WIDTH * 1:1] sv2v_tmp_A1F33;
	assign sv2v_tmp_A1F33 = addend_shamt;
	always @(*) mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH] = sv2v_tmp_A1F33;
	wire [1:1] sv2v_tmp_6A24C;
	assign sv2v_tmp_6A24C = sticky_before_add;
	always @(*) mid_pipe_sticky_q[0] = sv2v_tmp_6A24C;
	wire [(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)) * 1:1] sv2v_tmp_34E28;
	assign sv2v_tmp_34E28 = sum;
	always @(*) mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))] = sv2v_tmp_34E28;
	wire [1:1] sv2v_tmp_9C379;
	assign sv2v_tmp_9C379 = final_sign;
	always @(*) mid_pipe_final_sign_q[0] = sv2v_tmp_9C379;
	wire [3:1] sv2v_tmp_6B67B;
	assign sv2v_tmp_6B67B = inp_pipe_rnd_mode_q[(0 >= NUM_INP_REGS ? NUM_INP_REGS : NUM_INP_REGS - NUM_INP_REGS) * 3+:3];
	always @(*) mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * 3+:3] = sv2v_tmp_6B67B;
	wire [1:1] sv2v_tmp_08378;
	assign sv2v_tmp_08378 = result_is_special;
	always @(*) mid_pipe_res_is_spec_q[0] = sv2v_tmp_08378;
	wire [((1 + EXP_BITS) + MAN_BITS) * 1:1] sv2v_tmp_F6409;
	assign sv2v_tmp_F6409 = special_result;
	always @(*) mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] = sv2v_tmp_F6409;
	wire [5:1] sv2v_tmp_339AA;
	assign sv2v_tmp_339AA = special_status;
	always @(*) mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? 0 : NUM_MID_REGS) * 5+:5] = sv2v_tmp_339AA;
	wire [1:1] sv2v_tmp_7259D;
	assign sv2v_tmp_7259D = inp_pipe_tag_q[NUM_INP_REGS];
	always @(*) mid_pipe_tag_q[0] = sv2v_tmp_7259D;
	wire [1:1] sv2v_tmp_FAFEF;
	assign sv2v_tmp_FAFEF = inp_pipe_mask_q[NUM_INP_REGS];
	always @(*) mid_pipe_mask_q[0] = sv2v_tmp_FAFEF;
	wire [1:1] sv2v_tmp_8CE3D;
	assign sv2v_tmp_8CE3D = inp_pipe_aux_q[NUM_INP_REGS];
	always @(*) mid_pipe_aux_q[0] = sv2v_tmp_8CE3D;
	wire [1:1] sv2v_tmp_C7159;
	assign sv2v_tmp_C7159 = inp_pipe_valid_q[NUM_INP_REGS];
	always @(*) mid_pipe_valid_q[0] = sv2v_tmp_C7159;
	assign inp_pipe_ready[NUM_INP_REGS] = mid_pipe_ready[0];
	genvar _gv_i_2;
	generate
		for (_gv_i_2 = 0; _gv_i_2 < NUM_MID_REGS; _gv_i_2 = _gv_i_2 + 1) begin : gen_inside_pipeline
			localparam i = _gv_i_2;
			wire reg_ena;
			assign mid_pipe_ready[i] = mid_pipe_ready[i + 1] | ~mid_pipe_valid_q[i + 1];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_valid_q[i + 1] <= 1'b0;
				else
					mid_pipe_valid_q[i + 1] <= (flush_i ? 1'b0 : (mid_pipe_ready[i] ? mid_pipe_valid_q[i] : mid_pipe_valid_q[i + 1]));
			assign reg_ena = (mid_pipe_ready[i] & mid_pipe_valid_q[i]) | reg_ena_i[NUM_INP_REGS + i];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_eff_sub_q[i + 1] <= 1'sb0;
				else
					mid_pipe_eff_sub_q[i + 1] <= (reg_ena ? mid_pipe_eff_sub_q[i] : mid_pipe_eff_sub_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= 1'sb0;
				else
					mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= (reg_ena ? mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * EXP_WIDTH+:EXP_WIDTH] : mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= 1'sb0;
				else
					mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= (reg_ena ? mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * EXP_WIDTH+:EXP_WIDTH] : mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= 1'sb0;
				else
					mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH] <= (reg_ena ? mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * EXP_WIDTH+:EXP_WIDTH] : mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * EXP_WIDTH+:EXP_WIDTH]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH] <= 1'sb0;
				else
					mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH] <= (reg_ena ? mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH] : mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_sticky_q[i + 1] <= 1'sb0;
				else
					mid_pipe_sticky_q[i + 1] <= (reg_ena ? mid_pipe_sticky_q[i] : mid_pipe_sticky_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))] <= 1'sb0;
				else
					mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))] <= (reg_ena ? mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))] : mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_final_sign_q[i + 1] <= 1'sb0;
				else
					mid_pipe_final_sign_q[i + 1] <= (reg_ena ? mid_pipe_final_sign_q[i] : mid_pipe_final_sign_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 3+:3] <= 3'b000;
				else
					mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 3+:3] <= (reg_ena ? mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * 3+:3] : mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 3+:3]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_res_is_spec_q[i + 1] <= 1'sb0;
				else
					mid_pipe_res_is_spec_q[i + 1] <= (reg_ena ? mid_pipe_res_is_spec_q[i] : mid_pipe_res_is_spec_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] <= 1'sb0;
				else
					mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] <= (reg_ena ? mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] : mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 5+:5] <= 1'sb0;
				else
					mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 5+:5] <= (reg_ena ? mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? i : NUM_MID_REGS - i) * 5+:5] : mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? i + 1 : NUM_MID_REGS - (i + 1)) * 5+:5]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_tag_q[i + 1] <= 1'b0;
				else
					mid_pipe_tag_q[i + 1] <= (reg_ena ? mid_pipe_tag_q[i] : mid_pipe_tag_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_mask_q[i + 1] <= 1'sb0;
				else
					mid_pipe_mask_q[i + 1] <= (reg_ena ? mid_pipe_mask_q[i] : mid_pipe_mask_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					mid_pipe_aux_q[i + 1] <= 1'b0;
				else
					mid_pipe_aux_q[i + 1] <= (reg_ena ? mid_pipe_aux_q[i] : mid_pipe_aux_q[i + 1]);
		end
	endgenerate
	assign effective_subtraction_q = mid_pipe_eff_sub_q[NUM_MID_REGS];
	assign exponent_product_q = mid_pipe_exp_prod_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH];
	assign exponent_difference_q = mid_pipe_exp_diff_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH];
	assign tentative_exponent_q = mid_pipe_tent_exp_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * EXP_WIDTH+:EXP_WIDTH];
	assign addend_shamt_q = mid_pipe_add_shamt_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * SHIFT_AMOUNT_WIDTH+:SHIFT_AMOUNT_WIDTH];
	assign sticky_before_add_q = mid_pipe_sticky_q[NUM_MID_REGS];
	assign sum_q = mid_pipe_sum_q[(((3 * PRECISION_BITS) + 3) >= 0 ? 0 : (3 * PRECISION_BITS) + 3) + ((0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * (((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3)))+:(((3 * PRECISION_BITS) + 3) >= 0 ? (3 * PRECISION_BITS) + 4 : 1 - ((3 * PRECISION_BITS) + 3))];
	assign final_sign_q = mid_pipe_final_sign_q[NUM_MID_REGS];
	assign rnd_mode_q = mid_pipe_rnd_mode_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * 3+:3];
	assign result_is_special_q = mid_pipe_res_is_spec_q[NUM_MID_REGS];
	assign special_result_q = mid_pipe_spec_res_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS];
	assign special_status_q = mid_pipe_spec_stat_q[(0 >= NUM_MID_REGS ? NUM_MID_REGS : NUM_MID_REGS - NUM_MID_REGS) * 5+:5];
	wire [LOWER_SUM_WIDTH - 1:0] sum_lower;
	wire [LZC_RESULT_WIDTH - 1:0] leading_zero_count;
	wire signed [LZC_RESULT_WIDTH:0] leading_zero_count_sgn;
	wire lzc_zeroes;
	reg [SHIFT_AMOUNT_WIDTH - 1:0] norm_shamt;
	reg signed [EXP_WIDTH - 1:0] normalized_exponent;
	wire [(3 * PRECISION_BITS) + 4:0] sum_shifted;
	reg [PRECISION_BITS:0] final_mantissa;
	reg [(2 * PRECISION_BITS) + 2:0] sum_sticky_bits;
	wire sticky_after_norm;
	reg signed [EXP_WIDTH - 1:0] final_exponent;
	assign sum_lower = sum_q[LOWER_SUM_WIDTH - 1:0];
	lzc #(
		.WIDTH(LOWER_SUM_WIDTH),
		.MODE(1)
	) i_lzc(
		.in_i(sum_lower),
		.cnt_o(leading_zero_count),
		.empty_o(lzc_zeroes)
	);
	assign leading_zero_count_sgn = $signed({1'b0, leading_zero_count});
	always @(*) begin : norm_shift_amount
		if (_sv2v_0)
			;
		if ((exponent_difference_q <= 0) || (effective_subtraction_q && (exponent_difference_q <= 2))) begin
			if ((((exponent_product_q - leading_zero_count_sgn) + 1) >= 0) && !lzc_zeroes) begin
				norm_shamt = (PRECISION_BITS + 2) + leading_zero_count;
				normalized_exponent = (exponent_product_q - leading_zero_count_sgn) + 1;
			end
			else begin
				norm_shamt = $unsigned(($signed(PRECISION_BITS) + 2) + exponent_product_q);
				normalized_exponent = 0;
			end
		end
		else begin
			norm_shamt = addend_shamt_q;
			normalized_exponent = tentative_exponent_q;
		end
	end
	assign sum_shifted = sum_q << norm_shamt;
	always @(*) begin : small_norm
		if (_sv2v_0)
			;
		{final_mantissa, sum_sticky_bits} = sum_shifted;
		final_exponent = normalized_exponent;
		if (sum_shifted[(3 * PRECISION_BITS) + 4]) begin
			{final_mantissa, sum_sticky_bits} = sum_shifted >> 1;
			final_exponent = normalized_exponent + 1;
		end
		else if (sum_shifted[(3 * PRECISION_BITS) + 3])
			;
		else if (normalized_exponent > 1) begin
			{final_mantissa, sum_sticky_bits} = sum_shifted << 1;
			final_exponent = normalized_exponent - 1;
		end
		else
			final_exponent = 1'sb0;
	end
	assign sticky_after_norm = |{sum_sticky_bits} | sticky_before_add_q;
	wire pre_round_sign;
	wire [EXP_BITS - 1:0] pre_round_exponent;
	wire [MAN_BITS - 1:0] pre_round_mantissa;
	wire [(EXP_BITS + MAN_BITS) - 1:0] pre_round_abs;
	wire [1:0] round_sticky_bits;
	wire of_before_round;
	wire of_after_round;
	wire uf_before_round;
	wire uf_after_round;
	wire result_zero;
	wire rounded_sign;
	wire [(EXP_BITS + MAN_BITS) - 1:0] rounded_abs;
	assign of_before_round = final_exponent >= ((2 ** EXP_BITS) - 1);
	assign uf_before_round = final_exponent == 0;
	assign pre_round_sign = final_sign_q;
	assign pre_round_exponent = (of_before_round ? (2 ** EXP_BITS) - 2 : $unsigned(final_exponent[EXP_BITS - 1:0]));
	assign pre_round_mantissa = (of_before_round ? {MAN_BITS {1'sb1}} : final_mantissa[MAN_BITS:1]);
	assign pre_round_abs = {pre_round_exponent, pre_round_mantissa};
	assign round_sticky_bits = (of_before_round ? 2'b11 : {final_mantissa[0], sticky_after_norm});
	fpnew_rounding #(.AbsWidth(EXP_BITS + MAN_BITS)) i_fpnew_rounding(
		.abs_value_i(pre_round_abs),
		.sign_i(pre_round_sign),
		.round_sticky_bits_i(round_sticky_bits),
		.rnd_mode_i(rnd_mode_q),
		.effective_subtraction_i(effective_subtraction_q),
		.abs_rounded_o(rounded_abs),
		.sign_o(rounded_sign),
		.exact_zero_o(result_zero)
	);
	assign uf_after_round = (rounded_abs[(EXP_BITS + MAN_BITS) - 1:MAN_BITS] == {(((EXP_BITS + MAN_BITS) - 1) >= MAN_BITS ? (((EXP_BITS + MAN_BITS) - 1) - MAN_BITS) + 1 : (MAN_BITS - ((EXP_BITS + MAN_BITS) - 1)) + 1) * 1 {1'sb0}}) || (((pre_round_abs[(EXP_BITS + MAN_BITS) - 1:MAN_BITS] == {(((EXP_BITS + MAN_BITS) - 1) >= MAN_BITS ? (((EXP_BITS + MAN_BITS) - 1) - MAN_BITS) + 1 : (MAN_BITS - ((EXP_BITS + MAN_BITS) - 1)) + 1) * 1 {1'sb0}}) && (rounded_abs[(EXP_BITS + MAN_BITS) - 1:MAN_BITS] == 1)) && ((round_sticky_bits != 2'b11) || (!sum_sticky_bits[(MAN_BITS * 2) + 4] && ((rnd_mode_q == 3'b000) || (rnd_mode_q == 3'b100)))));
	assign of_after_round = rounded_abs[(EXP_BITS + MAN_BITS) - 1:MAN_BITS] == {(((EXP_BITS + MAN_BITS) - 1) >= MAN_BITS ? (((EXP_BITS + MAN_BITS) - 1) - MAN_BITS) + 1 : (MAN_BITS - ((EXP_BITS + MAN_BITS) - 1)) + 1) * 1 {1'sb1}};
	wire [WIDTH - 1:0] regular_result;
	wire [4:0] regular_status;
	assign regular_result = {rounded_sign, rounded_abs};
	assign regular_status[4] = 1'b0;
	assign regular_status[3] = 1'b0;
	assign regular_status[2] = of_before_round | of_after_round;
	assign regular_status[1] = uf_after_round & regular_status[0];
	assign regular_status[0] = (|round_sticky_bits | of_before_round) | of_after_round;
	wire [((1 + EXP_BITS) + MAN_BITS) - 1:0] result_d;
	wire [4:0] status_d;
	assign result_d = (result_is_special_q ? special_result_q : regular_result);
	assign status_d = (result_is_special_q ? special_status_q : regular_status);
	reg [(0 >= NUM_OUT_REGS ? ((1 - NUM_OUT_REGS) * ((1 + EXP_BITS) + MAN_BITS)) + ((NUM_OUT_REGS * ((1 + EXP_BITS) + MAN_BITS)) - 1) : ((NUM_OUT_REGS + 1) * ((1 + EXP_BITS) + MAN_BITS)) - 1):(0 >= NUM_OUT_REGS ? NUM_OUT_REGS * ((1 + EXP_BITS) + MAN_BITS) : 0)] out_pipe_result_q;
	reg [(0 >= NUM_OUT_REGS ? ((1 - NUM_OUT_REGS) * 5) + ((NUM_OUT_REGS * 5) - 1) : ((NUM_OUT_REGS + 1) * 5) - 1):(0 >= NUM_OUT_REGS ? NUM_OUT_REGS * 5 : 0)] out_pipe_status_q;
	reg [0:NUM_OUT_REGS] out_pipe_tag_q;
	reg [0:NUM_OUT_REGS] out_pipe_mask_q;
	reg [0:NUM_OUT_REGS] out_pipe_aux_q;
	reg [0:NUM_OUT_REGS] out_pipe_valid_q;
	wire [0:NUM_OUT_REGS] out_pipe_ready;
	wire [((1 + EXP_BITS) + MAN_BITS) * 1:1] sv2v_tmp_0F05D;
	assign sv2v_tmp_0F05D = result_d;
	always @(*) out_pipe_result_q[(0 >= NUM_OUT_REGS ? 0 : NUM_OUT_REGS) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] = sv2v_tmp_0F05D;
	wire [5:1] sv2v_tmp_16CBA;
	assign sv2v_tmp_16CBA = status_d;
	always @(*) out_pipe_status_q[(0 >= NUM_OUT_REGS ? 0 : NUM_OUT_REGS) * 5+:5] = sv2v_tmp_16CBA;
	wire [1:1] sv2v_tmp_1CCC3;
	assign sv2v_tmp_1CCC3 = mid_pipe_tag_q[NUM_MID_REGS];
	always @(*) out_pipe_tag_q[0] = sv2v_tmp_1CCC3;
	wire [1:1] sv2v_tmp_D6E81;
	assign sv2v_tmp_D6E81 = mid_pipe_mask_q[NUM_MID_REGS];
	always @(*) out_pipe_mask_q[0] = sv2v_tmp_D6E81;
	wire [1:1] sv2v_tmp_F4A83;
	assign sv2v_tmp_F4A83 = mid_pipe_aux_q[NUM_MID_REGS];
	always @(*) out_pipe_aux_q[0] = sv2v_tmp_F4A83;
	wire [1:1] sv2v_tmp_E45E7;
	assign sv2v_tmp_E45E7 = mid_pipe_valid_q[NUM_MID_REGS];
	always @(*) out_pipe_valid_q[0] = sv2v_tmp_E45E7;
	assign mid_pipe_ready[NUM_MID_REGS] = out_pipe_ready[0];
	genvar _gv_i_3;
	generate
		for (_gv_i_3 = 0; _gv_i_3 < NUM_OUT_REGS; _gv_i_3 = _gv_i_3 + 1) begin : gen_output_pipeline
			localparam i = _gv_i_3;
			wire reg_ena;
			assign out_pipe_ready[i] = out_pipe_ready[i + 1] | ~out_pipe_valid_q[i + 1];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_valid_q[i + 1] <= 1'b0;
				else
					out_pipe_valid_q[i + 1] <= (flush_i ? 1'b0 : (out_pipe_ready[i] ? out_pipe_valid_q[i] : out_pipe_valid_q[i + 1]));
			assign reg_ena = (out_pipe_ready[i] & out_pipe_valid_q[i]) | reg_ena_i[(NUM_INP_REGS + NUM_MID_REGS) + i];
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_result_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] <= 1'sb0;
				else
					out_pipe_result_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] <= (reg_ena ? out_pipe_result_q[(0 >= NUM_OUT_REGS ? i : NUM_OUT_REGS - i) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS] : out_pipe_result_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_status_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * 5+:5] <= 1'sb0;
				else
					out_pipe_status_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * 5+:5] <= (reg_ena ? out_pipe_status_q[(0 >= NUM_OUT_REGS ? i : NUM_OUT_REGS - i) * 5+:5] : out_pipe_status_q[(0 >= NUM_OUT_REGS ? i + 1 : NUM_OUT_REGS - (i + 1)) * 5+:5]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_tag_q[i + 1] <= 1'b0;
				else
					out_pipe_tag_q[i + 1] <= (reg_ena ? out_pipe_tag_q[i] : out_pipe_tag_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_mask_q[i + 1] <= 1'sb0;
				else
					out_pipe_mask_q[i + 1] <= (reg_ena ? out_pipe_mask_q[i] : out_pipe_mask_q[i + 1]);
			always @(posedge clk_i or negedge rst_ni)
				if (!rst_ni)
					out_pipe_aux_q[i + 1] <= 1'b0;
				else
					out_pipe_aux_q[i + 1] <= (reg_ena ? out_pipe_aux_q[i] : out_pipe_aux_q[i + 1]);
		end
	endgenerate
	assign out_pipe_ready[NUM_OUT_REGS] = out_ready_i;
	assign result_o = out_pipe_result_q[(0 >= NUM_OUT_REGS ? NUM_OUT_REGS : NUM_OUT_REGS - NUM_OUT_REGS) * ((1 + EXP_BITS) + MAN_BITS)+:(1 + EXP_BITS) + MAN_BITS];
	assign status_o = out_pipe_status_q[(0 >= NUM_OUT_REGS ? NUM_OUT_REGS : NUM_OUT_REGS - NUM_OUT_REGS) * 5+:5];
	assign extension_bit_o = 1'b1;
	assign tag_o = out_pipe_tag_q[NUM_OUT_REGS];
	assign mask_o = out_pipe_mask_q[NUM_OUT_REGS];
	assign aux_o = out_pipe_aux_q[NUM_OUT_REGS];
	assign out_valid_o = out_pipe_valid_q[NUM_OUT_REGS];
	assign busy_o = |{inp_pipe_valid_q, mid_pipe_valid_q, out_pipe_valid_q};
	generate
		if (NUM_OUT_REGS > 0) begin : genblk4
			assign early_out_valid_o = |{out_pipe_valid_q[NUM_OUT_REGS] & ~out_pipe_ready[NUM_OUT_REGS], out_pipe_valid_q[NUM_OUT_REGS - 1]};
		end
		else if (NUM_MID_REGS > 0) begin : genblk4
			assign early_out_valid_o = |{mid_pipe_valid_q[NUM_MID_REGS] & ~mid_pipe_ready[NUM_OUT_REGS], mid_pipe_valid_q[NUM_MID_REGS - 1]};
		end
		else if (NUM_INP_REGS > 0) begin : genblk4
			assign early_out_valid_o = |{inp_pipe_valid_q[NUM_INP_REGS] & ~inp_pipe_ready[NUM_INP_REGS], inp_pipe_valid_q[NUM_INP_REGS - 1]};
		end
		else begin : genblk4
			assign early_out_valid_o = 1'b0;
		end
	endgenerate
	initial _sv2v_0 = 0;
endmodule
