module top();
logic clk, q, d;
always_ff @(posedge clk) begin
    q = d;
end
endmodule
