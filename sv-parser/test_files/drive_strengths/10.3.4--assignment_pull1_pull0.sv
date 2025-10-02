/*
:name: cont_assignment_strength_pull1_pull0
:description: pull1 pull0 assignment test
:tags: 10.3.4
*/
module top(input a, input b);
wire (pull1, pull0) w = a & b;
endmodule