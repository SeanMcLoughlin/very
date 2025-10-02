/*
:name: changing_gclk_function
:description: $changing_gclk test
:tags: 20.13 16.9
:type: simulation elaboration parsing
*/
module top();
logic a, clk;
global clocking @(posedge clk); endclocking
assert property (@(posedge clk) $changing_gclk(a)) else $info;
endmodule