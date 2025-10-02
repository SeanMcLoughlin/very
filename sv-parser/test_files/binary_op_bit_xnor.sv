/*
:name: binary_op_bit_xnor
:description: ~^ operator test
:tags: 11.4.8
*/
module top();
int a = 12;
int b = 5;
int c;
initial begin
    c = a ~^ b;
end
endmodule
