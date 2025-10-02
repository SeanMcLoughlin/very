/*
:name: cont_assignment_strength_weak1_weak0
:description: weak1 weak0 assignment test
:tags: 10.3.4
*/
module top(input a, input b);
wire (weak1, weak0) w = a & b;
endmodule