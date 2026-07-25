pub const FileParser = @import("FileParser.zig");
pub const scanner = @import("scanner/root.zig");

test {
    _ = @import("FileParser.zig");
    _ = @import("scanner/root.zig");
}
