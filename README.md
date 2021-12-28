# lust: Lua in Rust

```bash
$ cargo build --release
$ cat test/fib.lua
function fib(n)
   if n < 2 then
      return n;
   end

   local n1 = fib(n-1);
   local n2 = fib(n-2);
   return n1 + n2;
end

print(fib(30));
$ time ./target/release/lust test/fib.lua
832040
./target/release/lust test/fib.lua  0.29s user 0.00s system 99% cpu 0.293 total
$ time lua test/fib.lua
832040
lua test/fib.lua  0.06s user 0.00s system 99% cpu 0.063 total
```