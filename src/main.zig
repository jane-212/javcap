const std = @import("std");
const Io = std.Io;
const Config = @import("Config.zig");

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;

    var rawConfig = try Config.init(alloc, io);
    defer rawConfig.deinit();
    const config = rawConfig.value;

    _ = config;
}
