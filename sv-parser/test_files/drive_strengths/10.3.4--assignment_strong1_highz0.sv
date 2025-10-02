/*
:name: cont_assignment_strength_strong1_highz0
:description: strong1 highz0 assignment test
:tags: 10.3.4
*/
module top(input a, input b);
wire (strong1, highz0) w = a & b;
endmodule