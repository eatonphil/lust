function i(n)
   if n < 12 then
      return n;
   end

   return i(n - 1);
end

print(i(1));
print(i(100));
