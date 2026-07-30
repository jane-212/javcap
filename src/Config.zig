const std = @import("std");
const Allocator = std.mem.Allocator;
const options = @import("options");
const Io = std.Io;
const domain = @import("domain");
const storage = domain.storage;
const known_folders = @import("known-folders");

pub const known_folders_config = known_folders.KnownFolderConfig{
    .xdg_on_mac = true,
};

pause_after_finish: bool,
worker_count: usize,
sources: []Source,

pub const Source = struct {
    type: storage.Type,
    from: []const u8,
    to: []const u8,
    exts: []const []const u8,

    fn validate(self: *const Source, alloc: Allocator, context: *Context) !void {
        if (!std.fs.path.isAbsolute(self.from)) {
            context.validate.field = try alloc.dupe(u8, "from");
            context.validate.value = try alloc.dupe(u8, self.from);
            context.validate.message = try alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
        if (!std.fs.path.isAbsolute(self.to)) {
            context.validate.field = try alloc.dupe(u8, "to");
            context.validate.value = try alloc.dupe(u8, self.to);
            context.validate.message = try alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
    }
};

const Self = @This();

pub fn init(alloc: Allocator, io: Io, context: *Context, env: *const std.process.Environ.Map) !Parsed(Self) {
    const path = try configPath(alloc, io, env);
    defer alloc.free(path);

    const file = Io.Dir.openFileAbsolute(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            try generateDefaultConfig(alloc, io, env);
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

    const config = try std.zon.parse.fromSliceAlloc(Self, alloc, configContentZ, &context.diagnostics, .{});
    errdefer std.zon.parse.free(alloc, config);

    try config.validate(alloc, context);

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
    validate: ValidateContext = .{},
    diagnostics: std.zon.parse.Diagnostics = .{},

    pub fn deinit(self: *Context, alloc: Allocator) void {
        if (self.validate.field) |field| alloc.free(field);
        if (self.validate.value) |value| alloc.free(value);
        if (self.validate.message) |message| alloc.free(message);
        self.diagnostics.deinit(alloc);
        self.* = undefined;
    }

    pub fn formatValidate(self: *const Context, writer: *Io.Writer) !void {
        try writer.print("{s}\n", .{self.validate.message.?});
        try writer.print("error: {s}: {s}\n", .{ self.validate.field.?, self.validate.value.? });
        try writer.flush();
    }

    pub fn formatDiagnostics(self: *const Context, writer: *Io.Writer) !void {
        try writer.print("配置文件解析失败\n", .{});
        const diagnostics = &self.diagnostics;
        var errors = diagnostics.iterateErrors();
        while (errors.next()) |err| {
            const msg = err.fmtMessage(diagnostics);
            try writer.print("error: {f}\n", .{msg});

            var notes = err.iterateNotes(diagnostics);
            while (notes.next()) |note| {
                const note_msg = note.fmtMessage(diagnostics);
                try writer.print("note: {f}\n", .{
                    note_msg,
                });
            }
        }
        try writer.flush();
    }
};

pub const ValidateContext = struct {
    field: ?[]const u8 = null,
    value: ?[]const u8 = null,
    message: ?[]const u8 = null,
};

fn validate(self: *const Self, alloc: Allocator, context: *Context) !void {
    for (self.sources) |source| try source.validate(alloc, context);
}

fn generateDefaultConfig(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) !void {
    const defaultConfig = @embedFile("config.zon");
    const dir = try configDir(alloc, io, env);
    defer alloc.free(dir);

    const cwd = Io.Dir.cwd();
    try cwd.createDirPath(io, dir);

    const path = try configPath(alloc, io, env);
    defer alloc.free(path);

    const file = try Io.Dir.createFileAbsolute(io, path, .{});
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileWriter = file.writer(io, &buffer);
    const writer = &fileWriter.interface;

    try writer.writeAll(defaultConfig);
    try writer.flush();
}

fn configPath(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) ![]const u8 {
    const dir = try configDir(alloc, io, env);
    defer alloc.free(dir);

    const path = try std.fs.path.join(alloc, &.{
        dir,
        "config.zon",
    });
    errdefer alloc.free(path);

    return path;
}

fn configDir(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) ![]const u8 {
    const root = try known_folders.getPath(io, alloc, env, .local_configuration) orelse return error.ConfigHomeNotFound;
    defer alloc.free(root);

    const dir = try std.fs.path.join(alloc, &.{
        root,
        options.name,
    });
    errdefer alloc.free(dir);

    return dir;
}

test "Source.validate — accepts absolute paths" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "/absolute/path/from",
        .to = "/absolute/path/to",
        .exts = &.{"mp4"},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try source.validate(alloc, &context);
}

test "Source.validate — rejects relative from path" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "relative/path",
        .to = "/absolute/path/to",
        .exts = &.{"mp4"},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try std.testing.expectError(error.ValidateFailed, source.validate(alloc, &context));
    try std.testing.expectEqualStrings("from", context.validate.field.?);
    try std.testing.expectEqualStrings("relative/path", context.validate.value.?);
    try std.testing.expectEqualStrings("路径必须是绝对路径", context.validate.message.?);
}

test "Source.validate — rejects relative to path" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "/absolute/path/from",
        .to = "relative/path",
        .exts = &.{"mp4"},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try std.testing.expectError(error.ValidateFailed, source.validate(alloc, &context));
    try std.testing.expectEqualStrings("to", context.validate.field.?);
    try std.testing.expectEqualStrings("relative/path", context.validate.value.?);
    try std.testing.expectEqualStrings("路径必须是绝对路径", context.validate.message.?);
}

test "Source.validate — reports from before to when both are relative" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "relative/from",
        .to = "relative/to",
        .exts = &.{"mp4"},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try std.testing.expectError(error.ValidateFailed, source.validate(alloc, &context));
    try std.testing.expectEqualStrings("from", context.validate.field.?);
}

test "Source.validate — does not allocate when paths are valid" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "/a",
        .to = "/b",
        .exts = &.{"mp4"},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try source.validate(alloc, &context);
    try std.testing.expect(context.validate.field == null);
    try std.testing.expect(context.validate.value == null);
    try std.testing.expect(context.validate.message == null);
}

test "Config.validate — accepts empty sources list" {
    const alloc = std.testing.allocator;
    const config = Self{
        .pause_after_finish = false,
        .sources = @as([]Source, &.{}),
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try config.validate(alloc, &context);
}

test "Config.validate — accepts valid config with multiple sources" {
    const alloc = std.testing.allocator;
    var sources = [_]Source{
        Source{ .type = .local, .from = "/a", .to = "/b", .exts = &.{"mp4"} },
        Source{ .type = .local, .from = "/c", .to = "/d", .exts = &.{ "mp4", "avi" } },
    };
    const config = Self{
        .pause_after_finish = false,
        .sources = &sources,
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try config.validate(alloc, &context);
}

test "Config.validate — rejects config with invalid first source" {
    const alloc = std.testing.allocator;
    var sources = [_]Source{
        Source{ .type = .local, .from = "relative", .to = "/b", .exts = &.{"mp4"} },
    };
    const config = Self{
        .pause_after_finish = false,
        .sources = &sources,
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try std.testing.expectError(error.ValidateFailed, config.validate(alloc, &context));
    try std.testing.expectEqualStrings("from", context.validate.field.?);
}

test "Config.validate — validates all sources, stops at first error" {
    const alloc = std.testing.allocator;
    var sources = [_]Source{
        Source{ .type = .local, .from = "/a", .to = "/b", .exts = &.{"mp4"} },
        Source{ .type = .local, .from = "/c", .to = "invalid", .exts = &.{"mp4"} },
        Source{ .type = .local, .from = "also/bad", .to = "/d", .exts = &.{"mp4"} },
    };
    const config = Self{
        .pause_after_finish = false,
        .sources = &sources,
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try std.testing.expectError(error.ValidateFailed, config.validate(alloc, &context));
    try std.testing.expectEqualStrings("to", context.validate.field.?);
    try std.testing.expectEqualStrings("invalid", context.validate.value.?);
}

test "Context.deinit — frees allocated fields" {
    const alloc = std.testing.allocator;
    var context: Context = .{};
    context.validate.field = try alloc.dupe(u8, "from");
    context.validate.value = try alloc.dupe(u8, "some/value");
    context.validate.message = try alloc.dupe(u8, "an error");

    context.deinit(alloc);
}

test "Context.deinit — handles null fields gracefully" {
    const alloc = std.testing.allocator;
    var context: Context = .{};

    context.deinit(alloc);
}

test "Context.deinit — partial cleanup (some fields set, some null)" {
    const alloc = std.testing.allocator;
    var context: Context = .{};
    context.validate.field = try alloc.dupe(u8, "field");

    context.deinit(alloc);
}

test "Source — empty exts slice is valid" {
    const alloc = std.testing.allocator;
    const source = Source{
        .type = .local,
        .from = "/a",
        .to = "/b",
        .exts = &.{},
    };

    var context: Context = .{};
    defer context.deinit(alloc);

    try source.validate(alloc, &context);
}
