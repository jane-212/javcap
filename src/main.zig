const std = @import("std");
const Io = std.Io;
const Config = @import("Config.zig");

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;

    var raw = try Config.init(alloc, io, "build.zig.zon");
    defer raw.deinit();
    const config = raw.value;

    _ = config;
}
