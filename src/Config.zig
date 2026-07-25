const std = @import("std");
const Allocator = std.mem.Allocator;
const builtin = @import("builtin");
const options = @import("options");
const Io = std.Io;
const domain = @import("domain");
const storage = domain.storage;

pause_after_finish: bool,
sources: []Source,

pub const Source = struct {
    type: storage.Type,
    from: []const u8,
    to: []const u8,
    exts: []const []const u8,

    fn validate(self: *const Source, context: *Context) !void {
        if (!std.fs.path.isAbsolute(self.from)) {
            context.validate.field = try context.alloc.dupe(u8, "from");
            context.validate.value = try context.alloc.dupe(u8, self.from);
            context.validate.message = try context.alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
        if (!std.fs.path.isAbsolute(self.to)) {
            context.validate.field = try context.alloc.dupe(u8, "to");
            context.validate.value = try context.alloc.dupe(u8, self.to);
            context.validate.message = try context.alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
    }
};

const Self = @This();

pub fn init(alloc: Allocator, io: Io, user: []const u8, context: *Context) !Parsed(Self) {
    const path = try configPath(alloc, user);
    defer alloc.free(path);

    const file = Io.Dir.openFileAbsolute(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            try generateDefaultConfig(alloc, io, user);
            return error.ConfigNotFound;
        },
        else => return err,
    };
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileReader = file.reader(io, &buffer);
    const reader = &fileReader.interface;

    const configContent = try reader.allocRemaining(alloc, .unlimited);
    defer alloc.free(configContent);

    const configContentZ = try alloc.dupeSentinel(u8, configContent, 0);
    defer alloc.free(configContentZ);

    const config = try std.zon.parse.fromSliceAlloc(Self, alloc, configContentZ, null, .{});
    errdefer std.zon.parse.free(alloc, config);

    try config.validate(context);

    return .{
        .alloc = alloc,
        .value = config,
    };
}

pub fn Parsed(comptime T: type) type {
    return struct {
        alloc: Allocator,
        value: T,

        pub fn deinit(self: *@This()) void {
            std.zon.parse.free(self.alloc, self.value);
            self.* = undefined;
        }
    };
}

pub const Context = struct {
    alloc: Allocator,
    validate: ValidateContext = .{},

    pub fn init(alloc: Allocator) Context {
        return .{
            .alloc = alloc,
        };
    }

    pub fn deinit(self: *Context) void {
        if (self.validate.field) |field| self.alloc.free(field);
        if (self.validate.value) |value| self.alloc.free(value);
        if (self.validate.message) |message| self.alloc.free(message);
        self.* = undefined;
    }
};

pub const ValidateContext = struct {
    field: ?[]const u8 = null,
    value: ?[]const u8 = null,
    message: ?[]const u8 = null,

    pub fn format(self: *ValidateContext, writer: *Io.Writer) !void {
        try writer.print("{s} => {s}: {s}\n", .{ self.message.?, self.field.?, self.value.? });
        try writer.flush();
    }
};

fn validate(self: *const Self, context: *Context) !void {
    for (self.sources) |source| try source.validate(context);
}

fn generateDefaultConfig(alloc: Allocator, io: Io, user: []const u8) !void {
    const defaultConfig = @embedFile("config.zon");
    const dir = try configDir(alloc, user);
    defer alloc.free(dir);

    const cwd = Io.Dir.cwd();
    try cwd.createDirPath(io, dir);

    const path = try configPath(alloc, user);
    defer alloc.free(path);

    const file = try Io.Dir.createFileAbsolute(io, path, .{});
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileWriter = file.writer(io, &buffer);
    const writer = &fileWriter.interface;

    try writer.writeAll(defaultConfig);
    try writer.flush();
}

fn configPath(alloc: Allocator, user: []const u8) ![]const u8 {
    const dir = try configDir(alloc, user);
    defer alloc.free(dir);

    const path = try std.fs.path.join(alloc, &.{
        dir,
        "config.zon",
    });
    errdefer alloc.free(path);

    return path;
}

fn configDir(alloc: Allocator, user: []const u8) ![]const u8 {
    const root = switch (builtin.os.tag) {
        .macos => "/Users",
        .linux => "/home",
        .windows => "C:\\Users",
        else => @compileError("Unsupported OS"),
    };

    const dir = try std.fs.path.join(alloc, &.{
        root,
        user,
        ".config",
        options.name,
    });
    errdefer alloc.free(dir);

    return dir;
}

test "Test the config path is right" {
    const alloc = std.testing.allocator;
    const expected = switch (builtin.os.tag) {
        .macos => "/Users/cat/.config/javcap/config.zon",
        .linux => "/home/cat/.config/javcap/config.zon",
        .windows => "C:\\Users\\cat\\.config\\javcap\\config.zon",
        else => @compileError("Unsupported OS"),
    };
    const actual = try configPath(alloc, "cat");
    defer alloc.free(actual);

    try std.testing.expectEqualStrings(expected, actual);
}

test "Test the config path is absolute" {
    const alloc = std.testing.allocator;
    const path = try configPath(alloc, "cat");
    defer alloc.free(path);

    try std.testing.expect(std.fs.path.isAbsolute(path));
}

test "Test the config directory is right" {
    const alloc = std.testing.allocator;
    const expected = switch (builtin.os.tag) {
        .macos => "/Users/cat/.config/javcap",
        .linux => "/home/cat/.config/javcap",
        .windows => "C:\\Users\\cat\\.config\\javcap",
        else => @compileError("Unsupported OS"),
    };
    const actual = try configDir(alloc, "cat");
    defer alloc.free(actual);

    try std.testing.expectEqualStrings(expected, actual);
}

test "Test the config directory is absolute" {
    const alloc = std.testing.allocator;
    const dir = try configDir(alloc, "cat");
    defer alloc.free(dir);

    try std.testing.expect(std.fs.path.isAbsolute(dir));
}
