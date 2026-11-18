class BohatDraw < Formula
  desc "Bohat Draw.io (RustFlow Draw) — Apple 90s Platinum diagramming engine with realtime co-op"
  homepage "https://github.com/Bohatman/rustflow-draw"
  url "https://github.com/Bohatman/rustflow-draw.git",
      tag:      "v0.1.0",
      revision: "22451fa465522083a2bb67458759876161119129"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/Bohatman/rustflow-draw.git", branch: "feat/phase-1-engine"

  depends_on "node"

  def install
    cd "apps/web" do
      system Formula["node"].opt_bin/"node", "build.mjs"
      libexec.install "server.mjs", "build.mjs", "package.json", "index.html", "dist", "src"
    end

    (bin/"bohat-draw").write <<~EOS
      #!/usr/bin/env bash
      set -euo pipefail
      export NODE_ENV="${NODE_ENV:-production}"
      export PORT="${PORT:-6129}"
      exec "#{Formula["node"].opt_bin}/node" "#{libexec}/server.mjs" "$@"
    EOS
    chmod 0755, bin/"bohat-draw"
  end

  service do
    run [opt_bin/"bohat-draw"]
    keep_alive true
    working_dir var
    log_path var/"log/bohat-draw.log"
    error_log_path var/"log/bohat-draw.log"
    environment_variables NODE_ENV: "production", PORT: "6129"
  end

  test do
    port = free_port
    pid = fork do
      ENV["PORT"] = port.to_s
      ENV["NODE_ENV"] = "production"
      exec bin/"bohat-draw"
    end
    sleep 2
    output = shell_output("curl -sf http://127.0.0.1:#{port}/healthz")
    assert_match '"status":"ok"', output
  ensure
    Process.kill("TERM", pid) if pid
  end
end
