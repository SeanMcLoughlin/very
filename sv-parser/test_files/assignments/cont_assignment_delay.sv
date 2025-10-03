module top(input a, input b);

wire w;

assign #10 w = a & b;

endmodule
