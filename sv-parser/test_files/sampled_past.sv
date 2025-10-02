/*
:name: past_function
:description: $past test
:tags: 20.13 16.9
:type: simulation elaboration parsing
*/
module top();
logic a, clk;
assert property (@(posedge clk) $past(a)) else $info;
endmodule