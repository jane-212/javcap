const std = @import("std");
const domain = @import("domain");
const Normal = @import("Normal.zig");
const Xml = @import("Xml.zig");

pub const WriterType = enum {
    normal,
    xml,
};

pub fn format(comptime t: WriterType, w: *std.Io.Writer, nfo: *const domain.Nfo) !void {
    switch (t) {
        .normal => try Normal.format(w, nfo),
        .xml => try Xml.format(w, nfo),
    }
}

test {
    _ = @import("Normal.zig");
    _ = @import("Xml.zig");
}
