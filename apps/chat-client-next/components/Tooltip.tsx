import { FC, ReactNode } from "react";
import styles from "./Tooltip.module.css";

export const Tooltip: FC<{ text: string; children: ReactNode }> = ({
  text,
  children,
}) => {
  return (
    <div className={styles.tooltip}>
      <div className={styles["tooltip-text"]}>{text}</div>
      {children}
    </div>
  );
};
