const std = @import("std");
const Allocator = std.mem.Allocator;

const Self = @This();

alloc: Allocator,

title: ?[]const u8 = null,
originalTitle: ?[]const u8 = null,
rating: ?f64 = null,
plot: ?[]const u8 = null,
runtime: ?u32 = null,
mpaa: ?Mpaa = null,
id: ?[]const u8 = null,
genres: std.ArrayList([]const u8),
tags: std.ArrayList([]const u8),
country: ?Country = null,
director: ?[]const u8 = null,
premiered: ?[]const u8 = null,
studio: ?[]const u8 = null,
poster: ?[]const u8 = null,
fanart: ?[]const u8 = null,
subtitle: ?[]const u8 = null,
actresses: std.ArrayList(Actress),

pub const Country = enum {
    cn,
    jp,
    us,
};

pub const Mpaa = enum {
    g,
    pg,
    pg13,
    r,
    nc17,
};

pub const Actress = struct {
    name: []const u8,
    thumb: ?[]const u8 = null,
};

pub fn init(alloc: Allocator) !Self {
    var genres = try std.ArrayList([]const u8).initCapacity(alloc, 8);
    errdefer genres.deinit(alloc);

    var tags = try std.ArrayList([]const u8).initCapacity(alloc, 8);
    errdefer tags.deinit(alloc);

    var actresses = try std.ArrayList(Actress).initCapacity(alloc, 8);
    errdefer actresses.deinit(alloc);

    return .{
        .alloc = alloc,
        .genres = genres,
        .tags = tags,
        .actresses = actresses,
    };
}

pub fn deinit(self: *Self) void {
    if (self.title) |s| self.alloc.free(s);
    if (self.originalTitle) |s| self.alloc.free(s);
    if (self.plot) |s| self.alloc.free(s);
    if (self.id) |s| self.alloc.free(s);
    if (self.director) |s| self.alloc.free(s);
    if (self.premiered) |s| self.alloc.free(s);
    if (self.studio) |s| self.alloc.free(s);
    if (self.poster) |s| self.alloc.free(s);
    if (self.fanart) |s| self.alloc.free(s);
    if (self.subtitle) |s| self.alloc.free(s);

    for (self.genres.items) |g| self.alloc.free(g);
    self.genres.deinit(self.alloc);

    for (self.tags.items) |t| self.alloc.free(t);
    self.tags.deinit(self.alloc);

    for (self.actresses.items) |a| {
        self.alloc.free(a.name);
        if (a.thumb) |t| self.alloc.free(t);
    }
    self.actresses.deinit(self.alloc);
}

pub fn merge(self: *Self, other: *Self) !void {
    const fields = @typeInfo(Self).@"struct".fields;
    inline for (fields) |field| {
        if (comptime std.mem.eql(u8, field.name, "alloc")) continue;

        const FT = field.type;
        if (comptime isOptional(FT)) {
            try mergeOptionalField(FT, &@field(self, field.name), @field(other, field.name), self.alloc);
        } else if (comptime isArrayList(FT)) {
            try mergeArrayListField(FT, &@field(self, field.name), &@field(other, field.name), self.alloc);
        }
    }
}

fn mergeOptionalField(comptime T: type, self_ptr: *T, other_val: T, alloc: Allocator) !void {
    if (self_ptr.* != null or other_val == null) return;

    const Child = @typeInfo(T).optional.child;
    if (comptime Child == []const u8) {
        self_ptr.* = try alloc.dupe(u8, other_val.?);
    } else {
        self_ptr.* = other_val;
    }
}

fn mergeArrayListField(comptime T: type, self_list: *T, other_list: *T, alloc: Allocator) !void {
    if (other_list.items.len == 0) return;
    try self_list.ensureUnusedCapacity(alloc, other_list.items.len);

    const ItemT = std.meta.Child(@TypeOf(self_list.items));
    for (other_list.items) |item| {
        if (comptime ItemT == []const u8) {
            if (containsString(self_list, item)) continue;
            self_list.appendAssumeCapacity(try alloc.dupe(u8, item));
        } else if (comptime ItemT == Actress) {
            if (containsActress(self_list, item.name)) continue;
            self_list.appendAssumeCapacity(.{
                .name = try alloc.dupe(u8, item.name),
                .thumb = if (item.thumb) |t| try alloc.dupe(u8, t) else null,
            });
        } else {
            @compileError("merge: unsupported ArrayList item type: " ++ @typeName(ItemT));
        }
    }
}

fn containsString(list: *std.ArrayList([]const u8), target: []const u8) bool {
    for (list.items) |existing| {
        if (std.mem.eql(u8, existing, target)) return true;
    }
    return false;
}

fn containsActress(list: *std.ArrayList(Actress), target_name: []const u8) bool {
    for (list.items) |existing| {
        if (std.mem.eql(u8, existing.name, target_name)) return true;
    }
    return false;
}

fn isOptional(comptime T: type) bool {
    return @typeInfo(T) == .optional;
}

fn isArrayList(comptime T: type) bool {
    if (@typeInfo(T) != .@"struct") return false;
    return @hasField(T, "items") and @hasDecl(T, "ensureUnusedCapacity") and @hasDecl(T, "appendAssumeCapacity");
}
