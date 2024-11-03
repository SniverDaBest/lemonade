# Different Error Types
Lemonade uses error codes. However, they can be kind of confusing. That's why there's this file.
## Emoticons
Lemonade uses different emoticons for different reasons. Here's a list of them:
- (X_X) - This is only shown when the kernel panicks, or something extremely bad happens.
- (0_0) - This is shown when something important fails, but won't crash the system.
- (-_-) - This is usually only shown when something like a random number generator fails, as it's not that important, and won't break anything.
- (^_^) - This is very rarely ever used, or even shown, as it's *usually* only used when I'm testing, and something suceeds, and I need to check. 
## Codes
Lemonade doesn't really have cryptic error codes, but just the errors thrown by functions. I don't believe that having things like `ERR_MEM_ALLOC` is very useful. It could mean anything. It could mean that it couldn't allocate memory for the kernel, which is a very, very bad thing, or it could just mean that it couldn't allocate some memory to generate a random number, which isn't that bad... However, here's how an error is formatted:
```
(X_X)\
\
Uh-oh! Lemonade panicked. Here's some info: panicked at src/[some_file]:[some_line]:[some_col]:
EXCEPTION: [some exception]
[insert details here...]
``` 
## Errors you may get
### Unable to initialize PICS
```
(X_X)

Uh-oh! Lemonade panicked. Here's some info: panicked at src/interrupts.rs:71:5:
EXCEPTION: DOUBLE FAULT
InterruptStackFrame {
    instruction_pointer: {
        0x8,
    },
    code_segment: 514,
    cpu_flags: 0x10000201e80,
    stack_pointer: VirtAddr(
        0x0,
    ),
    stack_segment: 0,
}
```

This error happens when the system is unable to initialize the PICS. Causes are unknown for this to happen naturally. However, by commenting out a line in `lib.rs`, it is possible to manually trigger it.