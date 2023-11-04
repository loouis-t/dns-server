# DNS-Server

A simple DNS server written in rust.  
Gives no DNS response, just a program to get familiar with rust.

## What it does

Not much. This program :

- listens for packets on port `53`,
- parses them and checks if the domain queried exists in cache.
- if not, sends a request to the configured DNS server (Cloudflare `1.1.1.3`) and adds the result to the cache.

## What I learned

If you don't have your own DNS server, you can lose up to `40ms` on each request.

That is to say, if we assume you make approximately 2 search requests per day, from age 20 to 80, you will lose
around `2 * 365 * 60 * 40 = 1 752 000ms` or `1 752s` or `29.2 minutes` in your life.

I'm glad this was helpful for you to learn.

(By the way, I have been learning Rust as well.)