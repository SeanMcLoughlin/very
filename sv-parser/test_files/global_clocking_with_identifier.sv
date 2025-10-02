module top;
    logic clk1, clk2;
    global clocking sys @(clk1 or clk2); endclocking
endmodule
