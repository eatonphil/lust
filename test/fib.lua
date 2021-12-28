function fib(n)
   if n < 2 then
      return n;
   end

   local n1 = fib(n-1);
   local n2 = fib(n-2);
   return n1 + n2;
end

print(fib(30));
