/*
:name: stable_gclk_function
:description: $stable_gclk test
:tags: 20.13 16.9
:type: simulation elaboration parsing
*/
module top();
logic a, clk;
global clocking @(posedge clk); endclocking
assert property (@(posedge clk) $stable_gclk(a)) else $info;
endmodule