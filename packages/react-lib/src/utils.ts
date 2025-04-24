const c = <T extends string | TemplateStringsArray>(className: T) => {
  return className;
};

const cn = c;

export { c, cn };
