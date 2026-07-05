use std::{
    fmt::{Display, Write},
    ops::Add,
};

use crate::gdb::rsp::Command::RemoveWriteWatchpoint;

mod query;

/// see: https://sourceware.org/gdb/current/onlinedocs/gdb.html/Standard-Replies.html#Standard-Replies
pub enum StandardReply<'a> {
    /// An empty response.
    Empty,
    /// An error with a two-digit hexadecimal error number.
    Error(u8),
    /// An error with an error message in ASCII.
    ErrorText(&'a str),
}

pub trait Packet {
    fn to_string(&self) -> String;
}

/// Several packets and replies include a thread-id field to identify a thread.
///
/// Normally these are positive numbers with a target-specific interpretation,
/// formatted as big-endian hex strings. A thread-id can also be a literal ‘-1’
/// to indicate all threads, or ‘0’ to pick any thread.
pub struct ThreadID(i32);

impl Display for ThreadID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == -1 {
            write!(f, "-1")
        } else if self.0 == 0 {
            write!(f, "0")
        } else {
            write!(f, "{:X}", self.0)
        }
    }
}

pub enum Address {
    Address32(u32),
    Address64(u64),
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Address32(addr) => write!(f, "{:X}", addr),
            Address::Address64(addr) => write!(f, "{:X}", addr),
        }
    }
}

/// The server must ignore ‘c’, ‘C’, ‘s’, ‘S’, and ‘r’ actions for threads that are already running. Conversely, the server must ignore ‘t’ actions for threads that are already stopped.
///
/// Note: In non-stop mode, a thread is considered running until GDB acknowledges an asynchronous stop notification for it with the ‘vStopped’ packet (see Remote Non-Stop).
///
/// The stub must support ‘vCont’ if it reports support for multiprocess extensions (see multiprocess extensions).
///
/// Reply: See Stop Reply Packets, for the reply specifications.
pub enum ContinueAction {
    /// Continue.
    Continue,
    /// Continue with signal sig. The signal sig should be two hex digits.
    ContinueSignal(u8), // tODO: signal type
    ///Step.
    Step,
    /// Step with signal sig. The signal sig should be two hex digits.
    StepSignal(u8), // TODO: signal type,
    /// Stop.
    ///
    /// The ‘t’ action is only relevant in non-stop mode (see Remote Non-Stop)
    /// and may be ignored by the stub otherwise. A stop reply should be
    /// generated for any affected thread not already stopped. When a thread is
    /// stopped by means of a ‘t’ action, the corresponding stop reply should
    /// indicate that the thread has stopped with signal ‘0’, regardless of whether
    /// the target uses some other signal as an implementation detail.
    Stop, // 't'
    /// Step once, and then keep stepping as long as the thread stops
    /// at addresses between start (inclusive) and end (exclusive).
    ///
    /// The remote stub reports a stop reply when either the thread goes
    /// out of the range or is stopped due to an unrelated reason, such
    /// as hitting a breakpoint. See range stepping.
    ///
    /// If the range is empty (start == end), then the action becomes
    /// equivalent to the ‘s’ action. In other words, single-step once,
    /// and report the stop (even if the stepped instruction jumps to start).
    ///
    /// (A stop reply may be sent at any point even if the PC is still within
    /// the stepping range; for example, it is valid to implement this packet
    /// in a degenerate way as a single instruction step operation.)
    StepRange { start: Address, end: Address },
}

impl ContinueAction {
    pub fn format(&self) -> String {
        match self {
            ContinueAction::Continue => "c".to_string(),
            ContinueAction::ContinueSignal(sig) => format!("C{:02x}", sig),
            ContinueAction::Step => "s".to_string(),
            ContinueAction::StepSignal(sig) => format!("S{:02x}", sig),
            ContinueAction::Stop => "t".to_string(),
            ContinueAction::StepRange { start, end } => format!("r{},{}", start, end),
        }
    }
}

pub enum NamedPacket {
    /// Attach to a new process with the specified process ID pid.
    ///
    /// The process ID is a hexadecimal integer identifying the process.
    ///
    /// In all-stop mode, all threads in the attached process are stopped;
    /// in non-stop mode, it may be attached without being stopped if
    /// that is supported by the target.
    ///
    /// This packet is only available in extended mode (see extended mode).
    ///
    /// Reply:
    ///
    /// ‘Any stop packet’ - for success in all-stop mode (see Stop Reply Packets)
    /// ‘OK’ - for success in non-stop mode (see Remote Non-Stop)
    Attach { pid: u8 }, // ‘vAttach;pid’
    Continue {
        action: Option<ContinueAction>, // tODO: type
        thread_id: Option<ThreadID>,
    }, // ‘vCont[;acion[:thread-id]]…’
    /// Request a list of actions supported by the ‘vCont’ packet.
    ///
    /// Reply: ‘vCont[;action…]’ - The ‘vCont’ packet is supported.
    ///  Each action is a supported command in the ‘vCont’ packet.
    QueryContinueActions,
    /// Interrupt remote target as if a control-C was pressed on the remote terminal.
    ///
    /// This is the equivalent to reacting to the ^C (‘\003’, the control-C character)
    /// character in all-stop mode while the target is running, except this works in
    /// non-stop mode. See interrupting remote targets, for more info on the all-stop variant.
    ///
    /// Reply: ‘OK’ - for success
    CtrlC, // vCtrlC
    // Perform a file operation on the target system.
    //
    // For details, see Host I/O Packets.
    // TODO: File, // ‘vFile:operation:parameter…’
    /// Direct the stub to erase length bytes of flash starting at addr.
    ///
    /// The region may enclose any number of flash blocks, but its start and
    /// end must fall on block boundaries, as indicated by the flash block
    /// size appearing in the memory map (see Memory Map Format). GDB groups
    /// flash memory programming operations together, and sends a ‘vFlashDone’
    /// request after each group; the stub is allowed to delay erase operation
    /// until the ‘vFlashDone’ packet is received.
    ///
    /// Reply: ‘OK’ - for success
    FlashErase {
        addr: Address,
        length: u32, // TODO: type?
    },
    /// Direct the stub to write data to flash address addr. The data is passed
    /// in binary form using the same encoding as for the ‘X’ packet (see Binary
    /// Data). The memory ranges specified by ‘vFlashWrite’ packets preceding a
    /// ‘vFlashDone’ packet must not overlap, and must appear in order of increasing
    /// addresses (although ‘vFlashErase’ packets for higher addresses may already
    /// have been received; the ordering is guaranteed only between ‘vFlashWrite’
    /// packets). If a packet writes to an address that was neither erased by a
    /// preceding ‘vFlashErase’ packet nor by some other target-specific method,
    /// the results are unpredictable.
    ///
    /// Reply: ‘OK’ - for success
    ///        ‘E.memtype’ - for vFlashWrite addressing non-flash memory
    FlashWrite { addr: Address, data: BinaryData },
    /// Indicate to the stub that flash programming operation is finished.
    ///
    /// The stub is permitted to delay or batch the effects of a group of
    /// ‘vFlashErase’ and ‘vFlashWrite’ packets until a ‘vFlashDone’ packet
    /// is received. The contents of the affected regions of flash memory
    /// are unpredictable until the ‘vFlashDone’ request is completed.
    FlashDone,
    /// Kill the process with the specified process ID pid, which is a hexadecimal
    /// integer identifying the process. This packet is used in preference to ‘k’
    /// when multiprocess protocol extensions are supported; see multiprocess extensions.
    ///
    /// Reply: ‘OK’ - for success
    Kill {
        pid: u8, // TODO: pid type
    }, // ‘vKill;pid’
    /// The correct reply to an unknown ‘v’ packet is to return the empty string, however,
    /// some older versions of gdbserver would incorrectly return ‘OK’ for unknown ‘v’ packets.
    ///
    /// The ‘vMustReplyEmpty’ is used as a feature test to check how gdbserver handles unknown
    /// packets, it is important that this packet be handled in the same way as other unknown
    /// ‘v’ packets. If this packet is handled differently to other unknown ‘v’ packets then
    /// it is possible that GDB may run into problems in other areas, specifically around use
    /// of ‘vFile:setfs:’.
    MustReplyEmpty, // ‘vMustReplyEmpty’
    /// Run the program filename, passing it each argument on its command line. The file and
    /// arguments are hex-encoded strings. If filename is an empty string, the stub may use
    /// a default program (e.g. the last program run). The program is created in the stopped state.
    ///
    /// If GDB sent the ‘single-inf-arg’ feature in the ‘qSupported’ packet (see single-inf-arg),
    /// and the stub replied with ‘single-inf-arg+’, then there will only be a single argument string,
    /// which includes all inferior arguments, separated with whitespace.
    ///
    /// This packet is only available in extended mode (see extended mode).
    ///
    /// Reply: ‘Any stop packet’ - for success (see Stop Reply Packets)
    Run {
        filename: String, // hex-encoded string
        // TODO: aryument should be list
        argument: Option<String>, // hex-encoded string
    }, // ‘vRun;filename[;argument]…’
    /// See Notification Packets.
    Stopped, // vStopped
}

impl Packet for NamedPacket {
    fn to_string(&self) -> String {
        match self {
            NamedPacket::Attach { pid } => format!("vAttach;{:x}", pid),
            NamedPacket::Continue { action, thread_id } => {
                if let Some(action) = action {
                    if let Some(thread_id) = thread_id {
                        format!("vCont;{}:{}", action.format(), thread_id)
                    } else {
                        format!("vCont;{}", action.format())
                    }
                } else {
                    format!("vCont")
                }
            }
            NamedPacket::QueryContinueActions => "vCont?".to_string(),
            NamedPacket::CtrlC => "vCtrlC".to_string(),
            NamedPacket::FlashErase { addr, length } => format!("vFlashErase;{},{}", addr, length),
            NamedPacket::FlashWrite { addr, data } => format!("vFlashWrite;{},{}", addr, data),
            NamedPacket::FlashDone => "vFlashDone".to_string(),
            NamedPacket::Kill { pid } => format!("vKill;{}", pid),
            NamedPacket::MustReplyEmpty => "vMustReplyEmpty".to_string(),
            NamedPacket::Run { filename, argument } => {
                let hex_filename: String = filename
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();

                if let Some(argument) = argument {
                    let hex_argument: String = argument
                        .as_bytes()
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();

                    // filename: hex-encoded string
                    format!("vRun{};{}", hex_filename, hex_argument)
                } else {
                    // filename: hex-encoded string
                    format!("vRun{}", hex_filename)
                }
            }
            NamedPacket::Stopped => "vStopped".to_string(),
        }
    }
}

/// The cond_list parameter is comprised of a series of expressions, concatenated
/// without separators. Each expression has the following form:
///
/// 'X len,expr'
///
/// len is the length of the bytecode expression.
/// expr is the actual conditional expression in bytecode form.
pub struct ConditionExpression {
    len: u32,      // tODO: type?
    expr: Vec<u8>, // todo: type?
}

impl Display for ConditionExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: not sure if this is the correct bytecode formatting:
        let bytecode: String = self.expr.iter().map(|b| format!("{:02X}", b)).collect();

        write!(f, "X{},{}", self.len, bytecode)
    }
}

/// The cond_list parameter is comprised of a series of expressions, concatenated
/// without separators. Each expression has the following form:
///
/// 'X len,expr'
///
/// len is the length of the bytecode expression.
/// expr is the actual conditional expression in bytecode form.
pub struct ConditionExpressionList(Vec<ConditionExpression>);

impl Display for ConditionExpressionList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // series of expressions, concatenated without separators.
        let c: String = self.0.iter().map(|expr| format!("{}", expr)).collect();
        write!(f, "{}", c)
    }
}

/// These breakpoint kinds are defined for the ‘Z0’ and ‘Z1’ packets.
///
/// see: https://sourceware.org/gdb/current/onlinedocs/gdb.html/ARM-Breakpoint-Kinds.html#ARM-Breakpoint-Kinds
#[repr(u8)]
#[derive(Clone)]
pub enum ARMBreakpointKind {
    /// 16-bit Thumb mode breakpoint.
    Thumb16Bit = 2,
    /// 32-bit Thumb mode (Thumb-2) breakpoint.
    Thumb32Bit = 3,
    /// 32-bit ARM mode breakpoint.
    ARM32Bit = 4,
}

/// These breakpoint kinds are defined for the ‘Z0’ and ‘Z1’ packets.
///
/// see: https://sourceware.org/gdb/current/onlinedocs/gdb.html/MIPS-Breakpoint-Kinds.html#MIPS-Breakpoint-Kinds
#[repr(u8)]
#[derive(Clone)]
pub enum MIPSBreakpointKind {
    MIPS16 = 2,
    MicroMIPS16Bit = 3,
    StandardMIPS32Bit = 4,
    MicroMIPS32Bit = 5,
}

pub enum BreakpointKind {
    ARM(ARMBreakpointKind),
    MIPS(MIPSBreakpointKind),
}

impl Display for BreakpointKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BreakpointKind::ARM(kind) => kind.clone() as u8,
                BreakpointKind::MIPS(kind) => kind.clone() as u8,
            }
        )
    }
}

/// Binary data in most packets is encoded as two hexadecimal digits per
/// byte of binary data. This allowed the traditional remote protocol
/// to work over connections which were only seven-bit clean. Some packets
/// designed more recently assume an eight-bit clean connection, and use a
/// more efficient encoding to send and receive binary data.
pub struct BinaryData(Vec<u8>);

impl Display for BinaryData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // encode as two hexadecimal digits per byte of binary data.
        let hex: String = self.0.iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "{}", hex)
    }
}

pub struct Argument(pub Vec<u8>);

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // encode as two hexadecimal digits per byte of binary data.
        let hex: String = self.0.iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "{}", hex)
    }
}

pub enum BreakpointMode {
    Set,   // S
    Clear, // C
}

impl Display for BreakpointMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakpointMode::Set => write!(f, "S"),
            BreakpointMode::Clear => write!(f, "C"),
        }
    }
}

#[repr(u8)]
#[derive(Clone)]
pub enum Signal {
    Stopped = 0,
}

impl Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}", self.clone() as u8)
    }
}

// TODO: should probably be another int type?
pub type Pid = u8;

/// Depending on the operation to be performed, op should be ‘c’ for
///  step and continue operations (note that this is deprecated, supporting
///  the ‘vCont’ command is a better option), and ‘g’ for other operations.
pub enum ThreadOperation {
    StepAndContinue, // c
    Other,           // g
}

impl Display for ThreadOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadOperation::StepAndContinue => write!(f, "c"),
            ThreadOperation::Other => write!(f, "g"),
        }
    }
}

pub enum Command {
    /// Enable extended mode. In extended mode, the remote server is made persistent.
    ///
    /// The ‘R’ packet is used to restart the program being debugged.
    ///
    /// Reply: "OK" - The remote target both supports and has enabled extended mode.
    EnableExtendedMode, // !

    /// This is sent when connection is first established
    /// to query the reason the target halted. The reply
    /// is the same as for step and continue. This packet
    /// has a special interpretation when the target is in
    /// non-stop mode; see Remote Non-Stop.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    Query, // ?

    /// Initialized argv[] array passed into program.
    ///
    /// arglen specifies the number of bytes in the hex encoded byte stream arg. See gdbserver for more details.
    ///
    /// Reply: "OK" - The arguments were set.
    Arguments { args: Vec<Argument> },
    /// Change the serial line speed to baud.
    ///
    /// (Don’t use this packet; its behavior is not well-defined.)
    Baud(u32),
    /// Set (mode is ‘S’) or clear (mode is ‘C’) a breakpoint at addr.
    ///
    /// Don't use this packet. Use the ‘Z’ and ‘z’ packets instead
    /// (see insert breakpoint or watchpoint packet).
    Breakpoint { addr: Address, mode: BreakpointMode },
    /// Execute the target system in reverse
    ///
    /// See Reverse Execution, for more information.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    BackwardContinue,
    /// Execute one instruction in reverse. No parameter.
    ///
    /// See Reverse Execution, for more information.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    BackwardSingleStep,
    /// Continue at addr, which is the address to resume. If addr is omitted, resume at current address.
    ///
    /// This packet is deprecated for multi-threading support. See vCont packet.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    Continue { addr: Option<Address> },
    /// Continue with signal sig (hex signal number). If ‘;addr’ is omitted, resume at same address.
    ///
    /// This packet is deprecated for multi-threading support. See vCont packet.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    ContinueWithSignal {
        /// hex signal number
        sig: Signal,
        addr: Option<Address>,
    },
    /// Toggle debug flag.
    ///
    /// Don’t use this packet; instead, define a general set packet (see General Query Packets).
    ToggleDebug,
    /// Used to detach GDB from the remote system. It is sent to the remote
    /// target before GDB disconnects via the detach command.
    ///
    /// Including a process ID is used when multiprocess protocol extensions
    /// are enabled (see multiprocess extensions), to detach only a specific process.
    Detach { pid: Option<Pid> },
    // TODO: ‘F RC,EE,CF;XX’ A reply from GDB to an ‘F’ packet sent by the target. This is part of the File-I/O protocol extension. See File-I/O Remote Protocol Extension, for the specification.
    /// Read general registers.
    ///
    /// Reply: ‘XX…’
    /// Each byte of register data is described by two hex digits.
    /// The bytes with the register are transmitted in target byte order.
    /// The size of each register and their position within the ‘g’ packet
    ///  are determined by the target description (see Target Descriptions); in
    ///  the absence of a target description, this is done using code internal to
    ///  GDB; typically this is some customary register layout for the architecture
    ///  in question.
    ///
    /// When reading registers, the stub may also return a string of literal ‘x’’s in
    ///  place of the register data digits, to indicate that the corresponding register’s
    ///  value is unavailable. For example, when reading registers from a trace frame (see
    /// Using the Collected Data), this means that the register has not been collected in
    ///  the trace frame. When reading registers from a live program, this indicates that
    ///  the stub has no means to access the register contents, even though the corresponding
    ///  register is known to exist. Note that if a register truly does not exist on the target,
    ///  then it is better to not include it in the target description in the first place.
    ///
    ///  For example, for an architecture with 4 registers of 4 bytes each, the following reply
    ///   indicates to GDB that registers 0 and 2 are unavailable, while registers 1 and 3 are
    ///   available, and both have zero value:
    ///    -> g
    ///    <- xxxxxxxx00000000xxxxxxxx00000000
    ReadGeneralRegisters,
    /// Write general registers. See read registers packet, for a description of the XX… data.
    ///
    /// Reply: ‘OK’ - for success
    WriteGeneralRegisters(BinaryData),
    /// Set thread for subsequent operations (‘m’, ‘M’, ‘g’, ‘G’, et.al.).
    ///
    /// Depending on the operation to be performed, op should be ‘c’ for
    ///  step and continue operations (note that this is deprecated, supporting
    ///  the ‘vCont’ command is a better option), and ‘g’ for other operations.
    /// The thread designator thread-id has the format and interpretation described in thread-id syntax.
    ///
    /// Reply: ‘OK’ - for success
    SetThread {
        op: ThreadOperation,
        thread_id: ThreadID,
    }, // H
    /// Step the remote target by a single clock cycle.
    ///
    /// If ‘,nnn’ is present, cycle step nnn cycles.
    /// If addr is present, cycle step starting at that address.
    SingleCycleStep {
        addr: Option<Address>,
        cycles: Option<u8>,
    }, // i
    /// Signal, then cycle step. See step with signal packet. See cycle step packet.
    SignalCycleStep, // I
    /// Kill request.
    ///
    /// The exact effect of this packet is not specified.
    ///
    /// For a bare-metal target, it may power cycle or reset the target system.
    /// For that reason, the ‘k’ packet has no reply.
    ///
    /// For a single-process target, it may kill that process if possible.
    ///
    /// A multiple-process target may choose to kill just one process, or all that are
    /// under GDB’s control. For more precise control, use the vKill packet (see vKill packet).
    ///
    /// If the target system immediately closes the connection in response to ‘k’, GDB does not
    /// consider the lack of packet acknowledgment to be an error, and assumes the kill was successful.
    ///
    /// If connected using target extended-remote, and the target does not close the connection in
    /// response to a kill request, GDB probes the target state as if a new connection was opened (see ? packet).
    Kill, // k
    /// Read length addressable memory units starting at address addr
    /// (see addressable memory unit). Note that addr does not have
    /// to be aligned to any particular boundary.
    ///
    /// The stub need not use any particular size or alignment when
    /// gathering data from memory for the response; even if addr is
    /// word-aligned and length is a multiple of the word size, the
    /// stub is free to use byte accesses, or not. For this reason, this
    /// packet may not be suitable for accessing memory-mapped I/O devices.
    ///
    /// Reply: ‘XX…’ Memory contents; each byte is transmitted as a two-digit
    /// hexadecimal number. The reply may contain fewer addressable memory units
    /// than requested if the server was reading from a trace frame memory and
    /// was able to read only part of the region of memory.
    ///
    /// Unlike most packets, this packet does not support ‘E.errtext’-style textual error
    /// replies (see textual error reply) by default. Stubs should be careful to only send
    /// such a reply if GDB reported support for it with the error-message feature (see error-message).
    ReadMemory {
        addr: Address,
        length: u8, // TODO: type,
    }, // m
    /// Write length addressable memory units starting at address addr (see addressable memory unit).
    /// The data is given by XX…; each byte is transmitted as a two-digit hexadecimal number.
    ///
    /// Reply: ‘OK’ - All the data was written successfully. (If only
    ///  part of the data was written, this command returns an error.)
    WriteMemory {
        addr: Address,
        length: u8,       // TODO: type,
        data: BinaryData, // TODO: type
    }, // M
    /// Read the value of register n; n is in hex. See read registers packet,
    /// for a description of how the returned register value is encoded.
    ///
    /// Reply: ‘XX…’ - the register’s value
    ReadRegister {
        register: u8, // n is in hex
    }, // p
    /// Write register n… with value r….
    ///
    /// The register number n is in hexadecimal, and r… contains
    /// two hex digits for each byte in the register (target byte order).
    ///
    /// Reply: ‘OK’ - for success
    WriteRegister {
        register: u8,      // n is in hex
        value: BinaryData, //  contains two hex digits for each byte in the register (target byte order).
    }, // P
    /// General query (‘q’) and set (‘Q’).
    ///
    /// These packets are described fully in General Query Packets.
    GeneralQuery(query::Query),
    /// Reset the entire system.
    ///
    /// Don’t use this packet; use the ‘R’ packet instead.
    Reset, // r
    /// Restart the program being debugged.
    ///
    /// The XX, while needed, is ignored. This packet is
    /// only available in extended mode (see extended mode).
    ///
    /// The ‘R’ packet has no reply.
    Restart(u8), // R
    /// Single step, resuming at addr. If addr is omitted, resume at same address.
    ///
    /// This packet is deprecated for multi-threading support. See vCont packet.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    SingleStep { addr: Option<Address> }, // s
    /// Step with signal. This is analogous to the ‘C’ packet, but requests a
    /// single-step, rather than a normal resumption of execution.
    ///
    /// This packet is deprecated for multi-threading support. See vCont packet.
    ///
    /// Reply: See Stop Reply Packets, for the reply specifications.
    StepWithSignal {
        signal: Signal,
        addr: Option<Address>,
    }, // S
    /// Search backwards starting at address addr for a match with pattern PP and
    /// mask MM, both of which are are 4 byte long. There must be at least 3 digits in addr.
    SearchBackwards {
        addr: Address,
        // TODO: not sure pattern and mask are correct
        pattern: u32,
        mask: u32,
    }, // t
    /// Find out if the thread thread-id is alive. See thread-id syntax.
    ///
    /// Reply: ‘OK’ - thread is still alive
    ThreadAlive { thread: ThreadID }, // T
    /// Packets starting with ‘v’ are identified by a multi-letter name,
    /// up to the first ‘;’ or ‘?’ (or the end of the packet).
    Named(NamedPacket), // v...
    /// Read length addressable memory units starting at address addr (see addressable memory
    /// unit). Note that addr does not have to be aligned to any particular boundary.
    ///
    /// The stub need not use any particular size or alignment when gathering data from memory
    /// for the response; even if addr is word-aligned and length is a multiple of the word
    /// size, the stub is free to use byte accesses, or not. For this reason, this packet may
    /// not be suitable for accessing memory-mapped I/O devices.
    ///
    /// GDB will only use this packet if the stub reports the ‘binary-upload’ feature is supported
    /// in its ‘qSupported’ reply (see qSupported).
    ///
    /// Reply:
    /// ‘b XX…’ - Memory contents as binary data (see Binary Data). The reply may contain fewer
    /// addressable memory units than requested if the server was reading from a trace frame
    /// memory and was able to read only part of the region of memory.
    /// ‘E NN’ - for an error
    ReadAddressableMemory { addr: Address, length: u32 },
    /// Write data to memory, where the data is transmitted in binary. Memory is specified
    /// by its address addr and number of addressable memory units length (see addressable
    /// memory unit); ‘XX…’ is binary data (see Binary Data).
    ///
    /// Reply: ‘OK’ - for success
    WriteAddressableMemory {
        addr: Address,
        length: u32,
        data: BinaryData,
    },
    ///
    RemoveBreakpoint {
        breakpoint_type: u8, // tODO: enum
        addr: Address,
        // hex-encoded
        breakpoint_kind: BreakpointKind, // tODO: enum
    }, // ‘z type,addr,kind’
    InsertBreakpoint {
        breakpoint_type: u8, // tODO: enum
        addr: Address,
        // hex-encoded
        breakpoint_kind: BreakpointKind, // tODO: enum
    }, // ‘Z type,addr,kind’
    /// Remove (‘z0’) a software breakpoint at address addr of type kind.
    ///
    /// A software breakpoint is implemented by replacing the instruction at addr with
    /// a software breakpoint or trap instruction. The kind is target-specific and
    /// typically indicates the size of the breakpoint in bytes that should be inserted.
    /// E.g., the ARM and MIPS can insert either a 2 or 4 byte breakpoint. Some
    /// architectures have additional meanings for kind (see Architecture-Specific Protocol
    /// Details); if no architecture-specific value is being used, it should be ‘0’. kind
    /// is hex-encoded.
    ///
    /// See also the ‘swbreak’ stop reason (see swbreak stop reason) for how to best report
    /// a software breakpoint event to GDB.
    RemoveSoftwareBreakpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // tODO: kind enum?
    },
    /// Insert (‘Z0’) a software breakpoint at address addr of type kind.
    ///
    /// A software breakpoint is implemented by replacing the instruction at addr with
    /// a software breakpoint or trap instruction. The kind is target-specific and
    /// typically indicates the size of the breakpoint in bytes that should be inserted.
    /// E.g., the ARM and MIPS can insert either a 2 or 4 byte breakpoint. Some
    /// architectures have additional meanings for kind (see Architecture-Specific Protocol
    /// Details); if no architecture-specific value is being used, it should be ‘0’. kind
    /// is hex-encoded.
    ///
    /// cond_list is an optional list of conditional expressions in bytecode form that should
    /// be evaluated on the target’s side. These are the conditions that should be taken into
    /// consideration when deciding if the breakpoint trigger should be reported back to GDB.
    ///
    /// See also the ‘swbreak’ stop reason (see swbreak stop reason) for how to best report
    /// a software breakpoint event to GDB.
    InsertSoftwareBreakpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // tODO: kind enum
        cond_list: Option<ConditionExpressionList>,
        /// If the flag is nonzero, then the breakpoint may remain active and the
        /// commands continue to be run even when GDB disconnects from the target.
        persist: u8,
        /// The optional cmd_list parameter introduces commands that may be run
        /// on the target, rather than being reported back to GDB.
        cmd_list: Option<ConditionExpressionList>,
    },
    /// Remove (‘z1’) a hardware breakpoint at address addr.
    ///
    /// A hardware breakpoint is implemented using a mechanism that is not
    /// dependent on being able to modify the target’s memory. The kind,
    /// cond_list, and cmd_list arguments have the same meaning as in ‘Z0’ packets.
    ///
    /// Implementation note: A hardware breakpoint is not affected by code movement.
    ///
    /// Reply: ‘OK’ - success
    RemoveHardwareBreakpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // tODO: kind enum
    }, // ‘z1,addr,kind’
    /// Insert (‘Z1’) a hardware breakpoint at address addr.
    ///
    /// A hardware breakpoint is implemented using a mechanism that is not
    /// dependent on being able to modify the target’s memory. The kind,
    /// cond_list, and cmd_list arguments have the same meaning as in ‘Z0’ packets.
    ///
    /// Implementation note: A hardware breakpoint is not affected by code movement.
    ///
    /// Reply: ‘OK’ - success
    InsertHardwareBreakpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind,                       // tODO: kind enum
        cond_list: Option<ConditionExpressionList>, // tODO: type
        /// If the flag is nonzero, then the breakpoint may remain active and the
        /// commands continue to be run even when GDB disconnects from the target.
        persist: u8,
        /// The optional cmd_list parameter introduces commands that may be run
        /// on the target, rather than being reported back to GDB.
        cmd_list: Option<ConditionExpressionList>,
    }, // ‘Z1,addr,kind[;cond_list…][;cmds:persist,cmd_list…]’
    /// Remove (‘z2’) a write watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    RemoveWriteWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, // ‘z2,addr,kind’
    /// Insert (‘Z2’) a write watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    InsertWriteWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, //‘Z2,addr,kind’
    /// Remove (‘z3’) a read watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    RemoveReadWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, // ‘z3,addr,kind’
    /// Insert (‘Z3’) a read watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    InsertReadWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, //‘Z3,addr,kind’
    /// Remove (‘z4’) an access watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    RemoveAccessWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, // ‘z4,addr,kind’
    /// Insert (‘Z4’) am access watchpoint at addr.
    ///
    /// The number of bytes to watch is specified by kind.
    InsertAccessWatchpoint {
        addr: Address,
        // hex-encoded
        kind: BreakpointKind, // TODO: kind
    }, //‘Z4,addr,kind’
}

impl Packet for Command {
    fn to_string(&self) -> String {
        match self {
            Command::EnableExtendedMode => "!".to_string(),
            Command::Query => "?".to_string(),
            Command::Arguments { args } => {
                let mut cmd: String = "A".to_string();

                let mut i = 0;
                for arg in args {
                    write!(cmd, "{},{},{}", arg.0.len(), i, arg).expect("successful write");
                    i = i + 1;
                }

                cmd
            }
            Command::Baud(baud) => format!("b{}", baud),
            Command::Breakpoint { addr, mode } => format!("B{},{}", addr, mode),
            Command::BackwardContinue => "bc".to_string(),
            Command::BackwardSingleStep => "bs".to_string(),
            Command::Continue { addr } => {
                format!("c{}", addr.as_ref().unwrap_or(&Address::Address32(0)))
            }
            Command::ContinueWithSignal { sig, addr } => {
                if let Some(addr) = addr {
                    format!("C{};{}", sig, addr)
                } else {
                    format!("C{}", sig)
                }
            }
            Command::ToggleDebug => "d".to_string(),
            Command::Detach { pid } => {
                if let Some(pid) = pid {
                    format!("D{:02X}", pid)
                } else {
                    "D".to_string()
                }
            }
            Command::ReadGeneralRegisters => "g".to_string(),
            Command::WriteGeneralRegisters(registers) => {
                format!("G{}", registers)
            }
            Command::SetThread { op, thread_id } => format!("H{}{}", op, thread_id),
            Command::SingleCycleStep { addr, cycles } => {
                if let Some(addr) = addr {
                    if let Some(cycles) = cycles {
                        format!("i{},{}", addr, cycles)
                    } else {
                        format!("i{}", addr)
                    }
                } else if let Some(cycles) = cycles {
                    format!("i,{}", cycles)
                } else {
                    "i".to_string()
                }
            }
            Command::SignalCycleStep => "I".to_string(),
            Command::Kill => "k".to_string(),
            Command::ReadMemory { addr, length } => format!("m{},{}", addr, length),
            Command::WriteMemory { addr, length, data } => {
                format!("M{},{}:{}", addr, length, data)
            }
            Command::ReadRegister { register } => format!("p{:X}", register),
            Command::WriteRegister { register, value } => {
                format!("P{:X}={}", register, value)
            }
            Command::GeneralQuery(query) => match query {
                _ => "".to_string(),
            },
            Command::Reset => "r".to_string(),
            Command::Restart(arg) => format!("R{:02x}", arg),
            Command::SingleStep { addr } => {
                // ‘s [addr]’
                if let Some(addr) = addr {
                    format!("s{}", addr)
                } else {
                    "s".to_string()
                }
            }
            Command::StepWithSignal { signal, addr } => {
                // ‘S sig[;addr]’
                if let Some(addr) = addr {
                    format!("S{};{}", signal, addr)
                } else {
                    format!("S{}", signal)
                }
            }
            Command::SearchBackwards {
                addr,
                pattern,
                mask,
            } => {
                // ‘t addr:PP,MM’
                format!("t{}:{:x},{:x}", addr, pattern, mask)
            }
            Command::ThreadAlive { thread } => {
                // ‘T thread-id’
                format!("T{}", thread)
            }
            Command::Named(named) => {
                format!("{}", named.to_string())
            }
            Command::ReadAddressableMemory { addr, length } => {
                // ‘x addr,length’
                format!("x{},{}", addr, length)
            }
            Command::WriteAddressableMemory { addr, length, data } => {
                // ‘X addr,length:XX…’
                format!("X{},{}:{}", addr, length, data)
            }
            Command::RemoveBreakpoint {
                breakpoint_type,
                addr,
                breakpoint_kind,
            } => {
                // ‘z type,addr,kind’
                format!("z{},{},{}", breakpoint_type, addr, breakpoint_kind)
            }
            Command::InsertBreakpoint {
                breakpoint_type,
                addr,
                breakpoint_kind,
            } => {
                // ‘Z type,addr,kind’
                format!("Z{},{},{}", breakpoint_type, addr, breakpoint_kind)
            }
            Command::RemoveSoftwareBreakpoint { addr, kind } => {
                // ‘z0,addr,kind’
                format!("z0,{},{}", addr, kind)
            }
            Command::InsertSoftwareBreakpoint {
                addr,
                kind,
                cond_list,
                persist,
                cmd_list,
            } => {
                // ‘Z0,addr,kind[;cond_list…][;cmds:persist,cmd_list…]’
                if let Some(cond_list) = cond_list {
                    if let Some(cmd_list) = cmd_list {
                        format!(
                            "Z0,{},{};{};cmds:{},{}",
                            addr, kind, cond_list, persist, cmd_list
                        )
                    } else {
                        format!("Z0,{},{};{}", addr, kind, cond_list)
                    }
                } else {
                    format!("Z0,{},{}", addr, kind)
                }
            }
            Command::RemoveHardwareBreakpoint { addr, kind } => {
                // ‘z1,addr,kind’
                format!("z1,{},{}", addr, kind)
            }
            Command::InsertHardwareBreakpoint {
                addr,
                kind,
                cond_list,
                persist,
                cmd_list,
            } => {
                // ‘Z1,addr,kind[;cond_list…][;cmds:persist,cmd_list…]’
                if let Some(cond_list) = cond_list {
                    if let Some(cmd_list) = cmd_list {
                        format!(
                            "Z1,{},{};{};cmds:{},{}",
                            addr, kind, cond_list, persist, cmd_list
                        )
                    } else {
                        format!("Z1,{},{};{}", addr, kind, cond_list)
                    }
                } else {
                    format!("Z1,{},{}", addr, kind)
                }
            }
            Command::RemoveWriteWatchpoint { addr, kind } => {
                // ‘z2,addr,kind’
                format!("z2,{},{}", addr, kind)
            }
            Command::InsertWriteWatchpoint { addr, kind } => {
                // ‘Z2,addr,kind’
                format!("Z2,{},{}", addr, kind)
            }
            Command::RemoveReadWatchpoint { addr, kind } => {
                // ‘z3,addr,kind’
                format!("z3,{},{}", addr, kind)
            }
            Command::InsertReadWatchpoint { addr, kind } => {
                // ‘Z3,addr,kind’
                format!("Z3,{},{}", addr, kind)
            }
            Command::RemoveAccessWatchpoint { addr, kind } => {
                // ‘z4,addr,kind’
                format!("z4,{},{}", addr, kind)
            }
            Command::InsertAccessWatchpoint { addr, kind } => {
                // ‘Z4,addr,kind’
                format!("Z4,{},{}", addr, kind)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_enable_extended_mode() {
        assert_eq!(Command::EnableExtendedMode.to_string(), "!")
    }

    #[test]
    fn test_command_query() {
        assert_eq!(Command::Query.to_string(), "?")
    }

    #[test]
    fn test_command_arguments() {
        assert_eq!(
            Command::Arguments {
                args: vec![Argument(vec![0, 1, 2, 3])]
            }
            .to_string(),
            "A4,0,00010203"
        )
    }

    #[test]
    fn test_command_baud() {
        assert_eq!(Command::Baud(152000).to_string(), "b152000")
    }

    #[test]
    fn test_command_breakpoint() {
        assert_eq!(
            Command::Breakpoint {
                addr: Address::Address32(64),
                mode: BreakpointMode::Clear
            }
            .to_string(),
            "B40,C"
        )
    }

    #[test]
    fn test_command_backwards_continue() {
        assert_eq!(Command::BackwardContinue.to_string(), "bc")
    }

    #[test]
    fn test_command_backwards_single_step() {
        assert_eq!(Command::BackwardSingleStep.to_string(), "bs")
    }

    #[test]
    fn test_command_continue() {
        assert_eq!(Command::Continue { addr: None }.to_string(), "c0");
        assert_eq!(
            Command::Continue {
                addr: Some(Address::Address32(64))
            }
            .to_string(),
            "c40"
        )
    }

    #[test]
    fn test_command_continue_with_signal() {
        assert_eq!(
            Command::ContinueWithSignal {
                sig: Signal::Stopped,
                addr: None
            }
            .to_string(),
            "C00"
        );
        assert_eq!(
            Command::ContinueWithSignal {
                sig: Signal::Stopped,
                addr: Some(Address::Address32(64))
            }
            .to_string(),
            "C00;40"
        )
    }

    #[test]
    fn test_command_toggle_debug() {
        assert_eq!(Command::ToggleDebug.to_string(), "d")
    }

    #[test]
    fn test_command_detach() {
        assert_eq!(Command::Detach { pid: None }.to_string(), "D");
        assert_eq!(Command::Detach { pid: Some(4) }.to_string(), "D04")
    }

    #[test]
    fn test_command_read_general_registers() {
        assert_eq!(Command::ReadGeneralRegisters.to_string(), "g");
    }

    #[test]
    fn test_command_write_general_register() {
        assert_eq!(
            Command::WriteGeneralRegisters(BinaryData(vec![0, 1, 2, 3])).to_string(),
            "G00010203"
        );
    }

    #[test]
    fn test_command_set_thread() {
        assert_eq!(
            Command::SetThread {
                op: ThreadOperation::StepAndContinue,
                thread_id: ThreadID(4)
            }
            .to_string(),
            "Hc4"
        );
    }

    #[test]
    fn test_single_cycle_step() {
        assert_eq!(
            Command::SingleCycleStep {
                addr: None,
                cycles: None
            }
            .to_string(),
            "i"
        );
        assert_eq!(
            Command::SingleCycleStep {
                addr: Some(Address::Address32(64)),
                cycles: Some(1)
            }
            .to_string(),
            "i40,1"
        );
        assert_eq!(
            Command::SingleCycleStep {
                addr: Some(Address::Address32(64)),
                cycles: None
            }
            .to_string(),
            "i40"
        );
    }

    #[test]
    fn test_command_signal_cycle_step() {
        assert_eq!(Command::SignalCycleStep.to_string(), "I")
    }

    #[test]
    fn test_command_kill() {
        assert_eq!(Command::Kill.to_string(), "k")
    }

    #[test]
    fn test_command_read_memory() {
        assert_eq!(
            Command::ReadMemory {
                addr: Address::Address32(64),
                length: 128
            }
            .to_string(),
            "m40,128"
        )
    }

    #[test]
    fn test_command_write_memory() {
        assert_eq!(
            Command::WriteMemory {
                addr: Address::Address32(64),
                length: 128,
                data: BinaryData(vec![0, 1, 2, 3])
            }
            .to_string(),
            "M40,128:00010203"
        )
    }

    #[test]
    fn test_command_read_register() {
        assert_eq!(Command::ReadRegister { register: 64 }.to_string(), "p40")
    }

    #[test]
    fn test_command_write_register() {
        assert_eq!(
            Command::WriteRegister {
                register: 64,
                value: BinaryData(vec![0, 1, 2, 3])
            }
            .to_string(),
            "P40=00010203"
        )
    }

    // TODO: GeneralQuery

    #[test]
    fn test_command_reset() {
        assert_eq!(Command::Reset.to_string(), "r")
    }

    #[test]
    fn test_command_restart() {
        assert_eq!(Command::Restart(64).to_string(), "R40")
    }

    #[test]
    fn test_command_single_step() {
        assert_eq!(Command::SingleStep { addr: None }.to_string(), "s");

        assert_eq!(
            Command::SingleStep {
                addr: Some(Address::Address32(64))
            }
            .to_string(),
            "s40"
        )
    }

    #[test]
    fn test_command_step_with_signal() {
        assert_eq!(
            Command::StepWithSignal {
                signal: Signal::Stopped,
                addr: None
            }
            .to_string(),
            "S00"
        );

        assert_eq!(
            Command::StepWithSignal {
                signal: Signal::Stopped,
                addr: Some(Address::Address32(64))
            }
            .to_string(),
            "S00;40"
        )
    }

    #[test]
    fn test_command_search_backwards() {
        assert_eq!(
            Command::SearchBackwards {
                addr: Address::Address32(128),
                pattern: 1,
                mask: 1
            }
            .to_string(),
            "t80:1,1"
        )
    }

    #[test]
    fn test_command_thread_alive() {
        assert_eq!(
            Command::ThreadAlive {
                thread: ThreadID(64)
            }
            .to_string(),
            "T40"
        )
    }

    // TODO: named

    #[test]
    fn test_command_read_addressable_memory() {
        assert_eq!(
            Command::ReadAddressableMemory {
                addr: Address::Address32(64),
                length: 128
            }
            .to_string(),
            "x40,128"
        )
    }

    #[test]
    fn test_command_write_addressable_memory() {
        assert_eq!(
            Command::WriteAddressableMemory {
                addr: Address::Address32(64),
                length: 4,
                data: BinaryData(vec![0, 1, 2, 3])
            }
            .to_string(),
            "X40,4:00010203"
        )
    }

    #[test]
    fn test_command_remove_breakpoint() {
        assert_eq!(
            Command::RemoveBreakpoint {
                breakpoint_type: 0,
                addr: Address::Address32(64),
                breakpoint_kind: BreakpointKind::ARM(ARMBreakpointKind::ARM32Bit)
            }
            .to_string(),
            "z0,40,4"
        )
    }

    #[test]
    fn test_command_insert_breakpoint() {
        assert_eq!(
            Command::InsertBreakpoint {
                breakpoint_type: 0,
                addr: Address::Address32(64),
                breakpoint_kind: BreakpointKind::ARM(ARMBreakpointKind::ARM32Bit)
            }
            .to_string(),
            "Z0,40,4"
        )
    }
}
