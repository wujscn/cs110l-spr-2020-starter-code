# DEET

## Basic milestones
- [x] Milestone 1: Run the inferior
- [x] Milestone 2: Stopping, resuming, and restarting the inferior
- [x] Milestone 3: Printing a backtrace
- [x] Milestone 4: Print stopped location
- [x] Milestone 5: Setting breakpoints
- [x] Milestone 6: Continuing from breakpoints
- [x] Milestone 7: Setting breakpoints on symbols
## Optional extensions
- [ ] Next line
- [ ] Print source code on stop
- [ ] Print variables

Since it might take a quite time to implement those extensions, so instead of finish them I decide to write a simple tutorial.

### Next line 
Keep the next line addr as our ending flag. Make a loop of using `ptrace::step` until the next line addr.
Before a `step`, check if it is a breakpoint addr. If it is, we restore the original bytes from 0xcc and modify to 0xcc again after this `step`.
After a `step`, print corresponding msg according to the waitpid status.

### Print source code on stop
Not difficult to read the source code and target the line with `DwarfData::get_line_from_addr`.

### Print variables
At first, we should use `DwarfData::get_function_from_addr` to indentify the current function name. Then we try to retrive variable name from the function local variable table and global variable table in order.
if it is a global var, we directly get the addr of it.
if it is a local var, we should get the stack pointer by using `ptrace::getregs`, and compute the real addr with the offset.
And the final problem I concern is that how to read variable with their address, and it is not a usual task. I found a lib called `vmemory` might be useful, waiting to figure out. 

## TODO
I forgot to impl setting breakpoint with line number in different file. 