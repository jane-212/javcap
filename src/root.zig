pub const Config = @import("Config.zig");

pub const known_folders_config = .{
    .xdg_on_mac = true,
};

test {
    _ = @import("Config.zig");
}
