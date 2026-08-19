type DustMascotProps = {
  className?: string;
  label?: string;
};

export function DustMascot({ className = "", label }: DustMascotProps) {
  return (
    <span
      className={`dust-mascot ${className}`.trim()}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      <img className="mascot-ferris" src="/ferris-happy.svg" alt="" />
      <img className="mascot-flutter" src="/flutter-logo.svg" alt="" />
      <img className="mascot-dart" src="/dart-logo.svg" alt="" />
    </span>
  );
}
