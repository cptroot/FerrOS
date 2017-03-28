
import gdb
import elftools
from elftools.elf.elffile import ELFFile

class ConnectCommand (gdb.Command):
    "Command for connecting to rusty-pintos."

    def __init__ (self):
        super (ConnectCommand, self).__init__("connect",
                gdb.COMMAND_RUNNING,
                gdb.COMPLETE_NONE)
# make sure that gdb knows we're not trying to debug 32 bit code
        gdb.execute( 'set architecture i386:x86-64:intel' )
# make sure gdb knows about the location of the rust library
        gdb.execute( 'directory $RUST_SRC_PATH' )

    def invoke (argument, from_tty, something):
# Remove the previous symbol tables
        gdb.execute( 'file' )
# Add debug.efi symbol table
        gdb.execute( 'file target/debug/debug.efi' )
# Use the symbol table to find what address the while loop has
        gdb.execute( 'info line gdb_stub.c:9' )
# Load the address into python
        offset = gdb.parse_and_eval('$_')
# Remove the symbol table
        gdb.execute( 'file' )

# Connect to the kernel
        gdb.execute( 'target remote localhost:1234' )
# Find the current address
        gdb.execute( 'x/1i $pc' )
        current = gdb.parse_and_eval('$_')
# Subtract and align to 8 bits
        base_address = ((current - offset) >> 8) << 8

# Use the elf file to load the correct offsets for text and data
        text_offset = 0
        data_offset = 0
        with open('target/debug/main.so', 'rb') as f:
            elffile = ELFFile(f)

            text_section = elffile.get_section_by_name(b'.text')
            data_section = elffile.get_section_by_name(b'.data')
            text_offset = text_section['sh_addr']
            data_offset = data_section['sh_addr']

# add the numbers together to get addresses to load symbols
        text_addr = base_address + text_offset
        data_addr = base_address + data_offset

# load the symbols
        gdb.execute( 'add-symbol-file target/debug/debug.efi 0x%x -s .data 0x%x' % (text_addr, data_addr) )

# unpause the program
        gdb.execute( 'set variable *(int *)($rbp - 0x4) = 0' )
# print "Linked with GDB"
        gdb.execute( 'n' )
        gdb.execute( 'n' )

# Determine if there's breakpoints set on rust_main. Add them otherwise

        gdb.Breakpoint('loader::run_kernel', internal=True, temporary=True)
        gdb.execute( 'c' )

# jump to the kernel
        gdb.execute( 'file target/debug/kernel.so' )
        gdb.Breakpoint('kernel::kernel_entry', internal=True, temporary=True)
        gdb.execute( 'c' )

class ConnectLoaderCommand (gdb.Command):
    "Command for connecting to rusty-pintos loader."

    def __init__ (self):
        super (ConnectLoaderCommand, self).__init__("connect_loader",
                gdb.COMMAND_RUNNING,
                gdb.COMPLETE_NONE)

    def invoke (argument, from_tty, something):
# Remove the previous symbol tables
        gdb.execute( 'file' )
# Add debug.efi symbol table
        gdb.execute( 'file target/debug/debug.efi' )
# Use the symbol table to find what address the while loop has
        gdb.execute( 'info line gdb_stub.c:9' )
# Load the address into python
        offset = gdb.parse_and_eval('$_')
# Remove the symbol table
        gdb.execute( 'file' )

# Connect to the kernel
        gdb.execute( 'target remote localhost:1234' )
# Find the current address
        gdb.execute( 'x/1i $pc' )
        current = gdb.parse_and_eval('$_')
# Subtract and align to 8 bits
        base_address = ((current - offset) >> 8) << 8

# Use the elf file to load the correct offsets for text and data
        text_offset = 0
        data_offset = 0
        with open('target/debug/main.so', 'rb') as f:
            elffile = ELFFile(f)

            text_section = elffile.get_section_by_name(b'.text')
            data_section = elffile.get_section_by_name(b'.data')
            text_offset = text_section['sh_addr']
            data_offset = data_section['sh_addr']

# add the numbers together to get addresses to load symbols
        text_addr = base_address + text_offset
        data_addr = base_address + data_offset

# load the symbols
        gdb.execute( 'add-symbol-file target/debug/debug.efi 0x%x -s .data 0x%x' % (text_addr, data_addr) )

# unpause the program
        gdb.execute( 'set variable *(int *)($rbp - 0x4) = 0' )
# print "Linked with GDB"
        gdb.execute( 'n' )
        gdb.execute( 'n' )

# Add a temporary breakpoint to rust_main and continue
        gdb.Breakpoint('rust_main', internal=True, temporary=True)
        gdb.execute( 'c' )

# Initialize the command
ConnectCommand()
ConnectLoaderCommand()
