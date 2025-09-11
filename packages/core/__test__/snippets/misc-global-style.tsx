import { Style } from '@flairjs/react';
import clsx from 'clsx';

export const Box = ({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) => {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
      <Style>
        {
          /*css*/ `
        .box {
          color: blue;
        }`
        }
      </Style>
      <Style global>
        {
          /*css*/ `
        .title {
          color: blue;
        }`
        }
      </Style>
    </div>
  )
}
