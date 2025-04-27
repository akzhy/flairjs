import clsx from "clsx";
import { Style } from "jsx-styled-react";
import { useState } from "react";

export const TestCaseComponent = () => {
  const [case5, setCase5] = useState(false);
  const [case8, setCase8] = useState(false);

  const case3 = "case-3";
  const case6_3 = "case-6-3";
  const case6_4 = "case-6-4";

  return (
    <>
      <p className="case-1">Case1</p>
      <p className="case-2 case-2-1 case-2-2">Case2</p>
      <div className={case3}>Case3</div>
      <div className={clsx("case-4")}>Case4</div>
      <button
        className={clsx("case-5", {
          "case-5-1": true,
          "case-5-2": case5,
          "case-5-3": true,
        })}
        onClick={() => setCase5((p) => !p)}
      >
        Case5
      </button>
      <div
        className={clsx("case-6", ["case-6-1", "case-6-2", case6_3], {
          [case6_4]: true,
        })}
      >
        Case6
      </div>
      <div className={["case-7", "case-7-1"].join(" ")}>Case7</div>
      <button
        className={case8 ? "case-8 case-8-1" : "case-8 case-8-2"}
        onClick={() => {
          setCase8((p) => !p);
        }}
      >
        Case8
      </button>
      <Style>{
        /*css*/ `
        .case-1 {
          color: rgb(255, 0, 1);
        }

        .case-2 {
          color: rgb(255, 0, 2);

          &.case-2-1 {
            background-color: rgb(255, 0, 2);
          }
        }

        .case-3 {
          color: rgb(255, 0, 3);
        }

        .case-4 {
          color: rgb(255, 0, 4);
        }

        .case-5 {
          color: rgb(255, 0, 5);

          &.case-5-1 {
            background-color: rgb(255, 0, 5);
          }

          &.case-5-2 {
            background-color: rgb(255, 0, 6);
          }
        }

        .case-6 {
          color: rgb(255, 0, 6);

          &.case-6-1 {
            background-color: rgb(255, 0, 6);
          }

          &.case-6-3 {
            border-color: rgb(255, 0, 6);
          }

          &.case-6-4 {
            outline-color: rgb(255, 0, 6);
          }
        }
      `
      }</Style>
    </>
  );
};
