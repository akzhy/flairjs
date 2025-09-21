import clsx from 'clsx'
import { c, cn, css } from '@flairjs/client'
import { useState } from 'react'

export const TestCaseComponent = () => {
  const [case5, setCase5] = useState(false)
  const [case8, setCase8] = useState(false)
  const [case9, setCase9] = useState(false)

  const case3 = 'case-3'
  const case6_3 = 'case-6-3'
  const case6_4 = 'case-6-4'

  const case9_1 = 'case-9 case-9-1'
  const case9_2 = 'case-9 case-9-2'

  const case10 = 'case-10'

  const case11_1 = 'case-11-1'
  const case11 = `case-11 ${case11_1} case-11-2`

  const case12_1 = 'case-12-1'
  const case12 = 'case-12 ' + case12_1 + ' case-12-2'

  const case13 = c('case-13 case-13-1')

  const case14 = cn('case-14 case-14-1')

  const case15Handler = () => {
    return c('case-15 case-15-1')
  }

  return (
    <>
      <p className="case-1">Case1</p>
      <p className="case-2 case-2-1 case-2-2">Case2</p>
      <div className={case3}>Case3</div>
      <div className={clsx('case-4')}>Case4</div>
      <button
        className={clsx('case-5', {
          'case-5-1': true,
          'case-5-2': case5,
          'case-5-3': true,
        })}
        onClick={() => setCase5((p) => !p)}
      >
        Case5
      </button>
      <div
        className={clsx('case-6', ['case-6-1', 'case-6-2', case6_3], {
          [case6_4]: true,
        })}
      >
        Case6
      </div>
      <div className={['case-7', 'case-7-1'].join(' ')}>Case7</div>
      <button
        className={case8 ? 'case-8 case-8-1' : 'case-8 case-8-2'}
        onClick={() => {
          setCase8((p) => !p)
        }}
      >
        Case8
      </button>
      <button
        className={case9 ? case9_1 : case9_2}
        onClick={() => {
          setCase9((p) => !p)
        }}
      >
        Case9
      </button>
      <div
        className={clsx({
          [case10]: true,
        })}
      >
        Case10
      </div>
      <div className={case11}>Case11</div>
      <div className={case12}>Case12</div>
      <div className={case13}>Case13</div>
      <div className={clsx(case14, 'case-14-2')}>Case14</div>
      <div className={case15Handler()}>Case15</div>
      <div className={`case-16 case-16-1`}>Case16</div>
    </>
  )
}

TestCaseComponent.flair = css`
.case-1 {
  color: $colors.red.100;
}

.case-2 {
  color: $colors.red.200;

  &.case-2-1 {
    background-color: $colors.red.200;
  }
}

.case-3 {
  color: $colors.red.300;
  padding: $spacing.4 $spacing.6;

  @media (min-width: 640px) {
    color: $colors.red.400;
  }
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

.case-7-1 {
  color: rgb(255, 0, 7);
}

.case-8 {
  &.case-8-1 {
    background-color: rgb(255, 0, 8);
  }

  &.case-8-2 {
    color: rgb(255, 0, 8);
  }
}

.case-9-1 {
  color: rgb(255, 0, 9);
}

.case-9-2 {
  background-color: rgb(255, 0, 9);
}

.case-10 {
  color: rgb(255, 0, 10);
}

.case-11-1 {
  color: rgb(255, 0, 11);
}

.case-11-2 {
  background-color: rgb(255, 0, 11);
}

.case-12-1 {
  color: rgb(255, 0, 12);
}

.case-12-2 {
  background-color: rgb(255, 0, 12);
}

.case-13 {
  color: rgb(255, 0, 13);
}

.case-14 {
  color: rgb(255, 0, 14);
}

.case-15 {
  color: rgb(255, 0, 15);
}

.case-16 {
  color: rgb(255, 0, 16);
}
`;
