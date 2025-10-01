module top();
wire [7:0] a = 8'b1101x001;
wire [7:0] b = 8'b1101x001;
wire c;
assign a = 8'b1101x001;
assign b = 8'b1101x001;
assign c = a == b;
final begin
    $display(":assert: ('%s' == '%d')", "x", c);
end
endmodule
