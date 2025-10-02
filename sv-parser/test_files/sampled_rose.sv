/*
:name: rose_function
:description: $rose test
:tags: 20.13 16.9
:type: simulation elaboration parsing
*/
module top();
logic a, clk;
assert property (@(posedge clk) $rose(a)) else $info;
endmodule