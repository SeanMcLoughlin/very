module top();
int a = 12;
int b = 5;
initial begin
    a = ~^b;
end
endmodule
