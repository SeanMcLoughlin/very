/*
:name: binary_op_mod
:description: % operator test
:tags: 11.4.3
*/
module top();
int a = 12;
int b = 5;
int c;
initial begin
    c = a % b;
end
endmodule
