module top;
    logic clk;
    global clocking my_clk @(posedge clk); endclocking : my_clk
endmodule
