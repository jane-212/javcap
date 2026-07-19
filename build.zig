const std = @import("std");
const zon = @import("build.zig.zon");
const builtin = @import("builtin");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const exe = b.addExecutable(.{
        .name = "javcap",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
            .strip = if (builtin.mode == .Debug) false else true,
        }),
    });
    b.installArtifact(exe);

    const options = b.addOptions();
    options.addOption([]const u8, "name", "javcap");
    options.addOption([]const u8, "version", zon.version);
    exe.root_module.addOptions("options", options);

    const domain = b.createModule(.{
        .root_source_file = b.path("src/domain/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("domain", domain);

    const core = b.createModule(.{
        .root_source_file = b.path("src/core/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("core", core);

    const spider = b.createModule(.{
        .root_source_file = b.path("src/spider/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    spider.addImport("core", core);
    spider.addImport("domain", domain);
    exe.root_module.addImport("spider", spider);

    const run_step = b.step("run", "Run the app");
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const test_step = b.step("test", "Run tests");
    const exe_tests = b.addTest(.{
        .root_module = exe.root_module,
    });
    const run_exe_tests = b.addRunArtifact(exe_tests);
    test_step.dependOn(&run_exe_tests.step);

    const check_step = b.step("check", "Check compile");
    check_step.dependOn(&run_cmd.step);
}
