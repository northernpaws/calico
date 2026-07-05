use std::fmt::Display;

pub enum AllowOperation {
    WriteReg,
    WriteMem,
    InsertBreak,
    InsertTrace,
    InsertFastTrace,
    Stop,
}

impl Display for AllowOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllowOperation::WriteReg => write!(f, "WriteReg"),
            AllowOperation::WriteMem => write!(f, "WriteMem"),
            AllowOperation::InsertBreak => write!(f, "InsertBreak"),
            AllowOperation::InsertTrace => write!(f, "InsertTrace"),
            AllowOperation::InsertFastTrace => write!(f, "InsertFastTrace"),
            AllowOperation::Stop => write!(f, "Stop"),
        }
    }
}

/// General query packets.
///
/// see: https://sourceware.org/gdb/current/onlinedocs/gdb.html/General-Query-Packets.html#General-Query-Packets
pub enum Query {
    /// Turn on or off the agent as a helper to perform some debugging
    /// operations delegated from GDB (see Control Agent).
    Agent(bool),
    /// Specify which operations GDB expects to request of the target,
    /// as a semicolon-separated list of operation name and value pairs.
    ///
    /// Possible values for op include ‘WriteReg’, ‘WriteMem’, ‘InsertBreak’,
    /// ‘InsertTrace’, ‘InsertFastTrace’, and ‘Stop’.
    ///
    /// val is either 0, indicating that GDB will not request the operation, or 1,
    /// indicating that it may. (The target can then use this to set up its own
    /// internals optimally, for instance if the debugger never expects to insert
    /// breakpoints, it may not need to install its own trap handler.)
    Allow { op: AllowOperation, val: bool },
    /// Return the current thread ID.
    ///
    /// Reply:
    /// ‘QC thread-id’ - Where thread-id is a thread ID as documented in thread-id syntax.
    /// ‘(anything else)’ - Any other reply implies the old thread ID.
    CurrentThreadID,
    /// Compute the CRC checksum of a block of memory using CRC-32 defined in IEEE 802.3. The
    /// CRC is computed byte at a time, taking the most significant bit of each byte first.
    /// The initial pattern code 0xffffffff is used to ensure leading zeros affect the CRC.
    ///
    /// Note: This is the same CRC used in validating separate debug files (see Debugging
    /// Information in Separate Files). However the algorithm is slightly different. When
    /// validating separate debug files, the CRC is computed taking the least significant
    /// bit of each byte first, and the final result is inverted to detect trailing zeros.
    ///
    /// Reply: ‘C crc32’ - The specified memory region’s checksum is crc32.
    ComputeCRC { addr: super::Address, length: u32 },
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Agent(enable) => {
                if *enable == true {
                    write!(f, "QAgent:1")
                } else {
                    write!(f, "QAgent:0")
                }
            }
            Query::Allow { op, val } => {
                if *val == true {
                    write!(f, "QAllow:{}:1", op)
                } else {
                    write!(f, "QAllow:{}:0", op)
                }
            }
            Query::CurrentThreadID => write!(f, "qC"),
            Query::ComputeCRC { addr, length } => write!(f, "qCRC:{},{}", addr, length),
        }
    }
}
