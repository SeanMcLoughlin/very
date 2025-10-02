module top();
wire [3:0] a = 3;
reg [3:0] b = 0;
initial begin
    unique0 case (a)
        0, 1: b = 1;
        2: b = 2;
        3: b = 3;
    endcase
end
endmodule
