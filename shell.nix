{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Frontend
    nodejs_20
    nodePackages.typescript
    
    # Backend (Rust)
    rustup
    pkg-config
    openssl
  ];
  
  # Переменные окружения для Rust
  shellHook = ''
    echo "🚀 WEB-KYPbEP окружение готово!"
    echo "📦 Node $(node --version), npm $(npm --version)"
    echo "🦀 Rust $(rustc --version 2>/dev/null || echo 'не установлен')"
    
    # Инициализируем Rust toolchain если нужно
    if ! command -v rustc &> /dev/null; then
      echo "⚙️  Инициализируем Rust..."
      rustup default stable
    fi
  '';
}