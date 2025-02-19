>[!WARNING]
Lemonade **DOES NOT** build at the moment. I'm working on things in [new-bootloader-testing](https://github.com/SniverDaBest/lemonade/tree/new-bootloader-testing). I'm going to get rid of the bootloader crate, and use GRand Unified Bootloader 2 (GRUB2) instead, as Rust had changes, which broke some critical parts of the OS.\
Don't worry, I'll *probably* get a working build... soon-*ish*. It'll be a while though.

# Info
Lemonade is an OS built in Rust, based off of blog_os, created by [Phil-Opp](https://github.com/phil-opp/blog_os). The blog he created helps people who love Rust, learn how to run it on bare metal, and make an OS. You can find it at <https://os.phil-opp.com/>. I would recommend it to people who want to get better at using Rust, and learning a `no_std` environment.\
\
Lemonade itself should NOT be used in production, and it probably never should. It's not even working much yet. It builds *(probably)*, but there are some bugs, and a lot of absolute garbage code that I wrote. *(note: I'm not a good programmer...)*

# Contributions
I would be very grateful if people would contribute. However, I know that probably won't happen much, or even at all... Also, I will try and look through issues/PRs on GitHub, that get opened.

# Notes
[![Builds](https://github.com/SniverDaBest/lemonade/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/SniverDaBest/lemonade/actions/workflows/rust.yml)
[![](https://tokei.rs/b1/github/SniverDaBest/lemonade)](https://github.com/SniverDaBest/lemonade)

Here are some notes for the users, about bugs, and other things that may happen.
1. You may get flooded with dots when running. That happens to me, and I don't know how to fix it.
2. True randomness is broken for some reason.
