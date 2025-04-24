import * as t from "@babel/types";
import { CSSModuleExports } from "lightningcss";
import { NodePath } from "@babel/traverse";

export const updateAttribute = ({
  node,
  attrName,
  classNameMap,
  path,
}: {
  node: t.JSXAttribute;
  attrName: string;
  classNameMap: CSSModuleExports;
  path: NodePath<t.JSXAttribute>;
}) => {
  if (node.value?.type === "StringLiteral") {
    const transformedClassNames = processStringLiteral(
      node.value,
      classNameMap
    );
    node.value = t.stringLiteral(transformedClassNames);
  } else if (node.value?.type === "JSXExpressionContainer") {
    if (node.value.expression.type === "CallExpression") {
      updateCallExpression(node.value.expression, classNameMap);
    } else if (node.value.expression.type === "LogicalExpression") {
      const transformedNode = updateLogicalExpression(
        node.value.expression,
        classNameMap
      );
      node.value.expression = transformedNode;
    } else if (node.value.expression.type === "ConditionalExpression") {
      const transformedNode = updateConditionalExpression(
        node.value.expression,
        classNameMap
      );
      node.value.expression = transformedNode;
    } else if (node.value.expression.type === "Identifier") {
      const varName = node.value.expression.name;
      const binding = path.scope.getBinding(varName);
      if (binding && t.isVariableDeclarator(binding.path.node)) {
        switch (binding.path.node.init?.type) {
          case "StringLiteral":
            const transformedClassNames = processStringLiteral(
              binding.path.node.init,
              classNameMap
            );
            binding.path.node.init.value = transformedClassNames;
            break;
          case "ObjectExpression":
            const properties = updateObjectExpression(
              binding.path.node.init,
              classNameMap
            );
            binding.path.node.init.properties = properties;
            break;
          case "ArrayExpression":
            const elements = updateArrayExpression(
              binding.path.node.init,
              classNameMap
            );
            binding.path.node.init.elements = elements;
            break;
          case "LogicalExpression":
            const transformedLogicalExpression = updateLogicalExpression(
              binding.path.node.init,
              classNameMap
            );
            binding.path.node.init = transformedLogicalExpression;
            break;
          case "ConditionalExpression":
            const transformedConditionalExpression =
              updateConditionalExpression(binding.path.node.init, classNameMap);
            binding.path.node.init = transformedConditionalExpression;
            break;
          default:
            break;
        }
      }
    }
  }
};

const processStringLiteral = (
  node: t.StringLiteral,
  classNameMap: CSSModuleExports
) => {
  const classNames = node.value.split(" ");
  const transformedClassNames = classNames.map((className) => {
    const exportName = classNameMap[className]?.name;
    return exportName ?? className;
  });

  return transformedClassNames.join(" ");
};

const updateObjectExpression = (
  node: t.ObjectExpression,
  classNameMap: CSSModuleExports
) => {
  const properties = node.properties.map((property) => {
    if (property.type === "ObjectProperty" && t.isStringLiteral(property.key)) {
      const exportName = classNameMap[property.key.value];
      if (exportName) {
        return t.objectProperty(
          t.stringLiteral(exportName.name),
          property.value
        );
      }
    }
    return property;
  });
  return properties;
};

const updateArrayExpression = (
  node: t.ArrayExpression,
  classNameMap: CSSModuleExports
) => {
  const elements: t.ArrayExpression["elements"] = node.elements.map(
    (element) => {
      switch (element?.type) {
        case "ObjectExpression":
          const properties = updateObjectExpression(element, classNameMap);
          element.properties = properties;
          return element;
        case "StringLiteral":
          const value = processStringLiteral(element, classNameMap);
          element.value = value;
          return element;
        case "ArrayExpression":
          const elements = updateArrayExpression(element, classNameMap);
          element.elements = elements;
          return element;
        case "LogicalExpression":
          return updateLogicalExpression(element, classNameMap);
        case "ConditionalExpression":
          return updateConditionalExpression(element, classNameMap);
        default:
          return element;
      }
    }
  );
  return elements;
};

const updateLogicalExpression = (
  node: t.LogicalExpression,
  classNameMap: CSSModuleExports
) => {
  switch (node.right.type) {
    case "StringLiteral":
      const transformedClassNames = processStringLiteral(
        node.right,
        classNameMap
      );
      node.right.value = transformedClassNames;
      return node;
    case "ObjectExpression":
      const properties = updateObjectExpression(node.right, classNameMap);
      node.right.properties = properties;
      return node;
    case "ArrayExpression":
      const elements = updateArrayExpression(node.right, classNameMap);
      node.right.elements = elements;
      return node;
    case "ConditionalExpression":
      return updateConditionalExpression(node.right, classNameMap);
    case "CallExpression":
      return updateCallExpression(node.right, classNameMap);
    case "LogicalExpression":
      return updateLogicalExpression(node.right, classNameMap);
    default:
      return node;
  }
};

const updateConditionalExpression = (
  node: t.ConditionalExpression,
  classNameMap: CSSModuleExports
) => {
  updateConditionalExpressionLeaf(node, "consequent", classNameMap);
  updateConditionalExpressionLeaf(node, "alternate", classNameMap);
  return node;
};

const updateConditionalExpressionLeaf = (
  node: t.ConditionalExpression,
  type: "consequent" | "alternate",
  classNameMap: CSSModuleExports
) => {
  switch (node[type].type) {
    case "StringLiteral":
      const transformedClassNames = processStringLiteral(
        node[type],
        classNameMap
      );
      node[type].value = transformedClassNames;
      return node;
    case "ObjectExpression":
      const properties = updateObjectExpression(node[type], classNameMap);
      node[type].properties = properties;
      return node;
    case "ArrayExpression":
      const elements = updateArrayExpression(node[type], classNameMap);
      node[type].elements = elements;
      return node;
    case "ConditionalExpression":
      return updateConditionalExpression(node[type], classNameMap);
    case "LogicalExpression":
      return updateLogicalExpression(node[type], classNameMap);
    case "CallExpression":
      return updateCallExpression(node[type], classNameMap);
    default:
      return node;
  }
};

export const updateCallExpression = (
  node: t.CallExpression,
  classNameMap: CSSModuleExports
) => {
  const args = node.arguments.map((arg) => {
    if (arg.type === "StringLiteral") {
      const transformedClassNames = processStringLiteral(arg, classNameMap);
      arg.value = transformedClassNames;
      return arg;
    } else if (arg.type === "ObjectExpression") {
      const properties = updateObjectExpression(arg, classNameMap);
      arg.properties = properties;
    } else if (arg.type === "ArrayExpression") {
      const transformedClassNames = updateArrayExpression(arg, classNameMap);
      arg.elements = transformedClassNames;
    } else if (arg.type === "LogicalExpression") {
      const transformedClassNames = updateLogicalExpression(arg, classNameMap);
      arg.right = transformedClassNames;
    } else if (arg.type === "ConditionalExpression") {
      const transformedClassNames = updateConditionalExpression(arg, classNameMap);
      arg.consequent = transformedClassNames;
    } else if (arg.type === "CallExpression") {
      updateCallExpression(arg, classNameMap);
    }

    return arg;
  });

  node.arguments = args;
  return node;
};
