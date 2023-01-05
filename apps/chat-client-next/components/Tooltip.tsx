import { FC, ReactNode } from "react";

export const Tooltip: FC<{ text: string; children: ReactNode }> = ({
  text,
  children,
}) => {
  return (
    <div className="tooltip">
      <div className="tooltip-text">{text}</div>
      {children}
    </div>
  );
};
