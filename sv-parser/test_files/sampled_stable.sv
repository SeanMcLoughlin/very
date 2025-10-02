/*
:name: stable_function
:description: $stable test
:tags: 20.13 16.9
:type: simulation elaboration parsing
*/
module top();
logic a, clk;
assert property (@(posedge clk) $stable(a)) else $info;
endmodule