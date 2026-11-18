class BohatDraw < Formula
  desc "Bohat Draw.io (RustFlow Draw) — Retro 90s Platinum desktop diagramming app with realtime co-op"
  homepage "https://github.com/Bohatman/rustflow-draw"
  url "https://github.com/Bohatman/rustflow-draw.git",
      tag: "v0.1.2"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/Bohatman/rustflow-draw.git", branch: "feat/phase-1-engine"

  depends_on "node"

  def install
    cd "apps/web" do
      system Formula["node"].opt_bin/"node", "build.mjs"
      libexec.install "server.mjs", "build.mjs", "package.json", "index.html", "dist", "src"
    end

    app_bin = ""
    if OS.mac? && File.exist?("apps/desktop/macos/BohatDrawApp.swift")
      app_contents = libexec/"Bohat Draw.app/Contents"
      (app_contents/"MacOS").mkpath
      system "swiftc", "-O", "apps/desktop/macos/BohatDrawApp.swift",
             "-framework", "Cocoa", "-framework", "WebKit",
             "-o", app_contents/"MacOS/BohatDrawApp"
      (app_contents/"Info.plist").write <<~PLIST
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
          <key>CFBundleName</key><string>Bohat Draw.io</string>
          <key>CFBundleIdentifier</key><string>com.bohatman.bohat-draw</string>
          <key>CFBundleVersion</key><string>#{version}</string>
          <key>CFBundleExecutable</key><string>BohatDrawApp</string>
          <key>NSHighResolutionCapable</key><true/>
        </dict>
        </plist>
      PLIST
      app_bin = (app_contents/"MacOS/BohatDrawApp").to_s
    end

    (bin/"bohat-draw").write <<~EOS
      #!/usr/bin/env bash
      set -euo pipefail
      NODE_BIN="#{Formula["node"].opt_bin}/node"
      SERVER_JS="#{libexec}/server.mjs"
      MAC_APP_BIN="#{app_bin}"

      if [ "${1:-}" = "--server" ]; then
        shift
        export NODE_ENV="${NODE_ENV:-production}"
        export PORT="${PORT:-6129}"
        exec "$NODE_BIN" "$SERVER_JS" "$@"
      fi

      PORT="$("$NODE_BIN" -e 'const net=require("net");const s=net.createServer();s.listen(0,"127.0.0.1",()=>{console.log(s.address().port);s.close();});')"
      export NODE_ENV="production"
      export PORT
      "$NODE_BIN" "$SERVER_JS" >/dev/null 2>&1 &
      SERVER_PID=$!
      trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT

      for _ in $(seq 1 40); do
        if curl -sf "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
          break
        fi
        sleep 0.1
      done

      URL="http://127.0.0.1:${PORT}"
      if [ -n "$MAC_APP_BIN" ] && [ -x "$MAC_APP_BIN" ]; then
        "$MAC_APP_BIN" "$URL"
        exit 0
      fi

      for browser in google-chrome chromium chromium-browser brave-browser microsoft-edge; do
        if command -v "$browser" >/dev/null 2>&1; then
          PROFILE_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/bohat-draw/app-profile"
          mkdir -p "$PROFILE_DIR"
          "$browser" --app="$URL" --class=bohat-draw --user-data-dir="$PROFILE_DIR" --new-window >/dev/null 2>&1
          exit 0
        fi
      done

      echo "No native WebView or Chromium app-mode browser found; running server at $URL"
      wait "$SERVER_PID"
    EOS
    chmod 0755, bin/"bohat-draw"
  end

  service do
    run [opt_bin/"bohat-draw", "--server"]
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
      exec bin/"bohat-draw", "--server"
    end
    sleep 2
    output = shell_output("curl -sf http://127.0.0.1:#{port}/healthz")
    assert_match '"status":"ok"', output
  ensure
    Process.kill("TERM", pid) if pid
  end
end
