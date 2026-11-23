class BohatDraw < Formula
  desc "Bohat Draw.io (RustFlow Draw) — 100% Rust desktop & WASM diagramming studio with realtime co-op"
  homepage "https://github.com/Bohatman/rustflow-draw"
  url "https://github.com/Bohatman/rustflow-draw.git",
      tag: "v0.2.0"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/Bohatman/rustflow-draw.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "apps/studio")
    (libexec/"dist-wasm").install Dir["apps/studio/dist/*"] if Dir.exist?("apps/studio/dist")
  end

  service do
    run [opt_bin/"bohat-draw", "--server"]
    keep_alive true
    working_dir var
    log_path var/"log/bohat-draw.log"
    error_log_path var/"log/bohat-draw.log"
    environment_variables PORT: "6129"
  end

  test do
    port = free_port
    pid = fork do
      ENV["PORT"] = port.to_s
      exec bin/"bohat-draw", "--server"
    end
    sleep 2
    output = shell_output("curl -sf http://127.0.0.1:#{port}/healthz")
    assert_match '"status":"ok"', output
  ensure
    Process.kill("TERM", pid) if pid
  end
end
