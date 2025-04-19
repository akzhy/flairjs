import { ReactNode } from "react";

export interface StyleProps {
  children?: ReactNode;
  global?: boolean;
}

export const Style = ({ children, global, ...props }: StyleProps) => {
  return <style {...props}>{children}</style>;
};
