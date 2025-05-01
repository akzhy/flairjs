import * as t from "@babel/types";
import { CSSModuleExports } from "lightningcss";
import { NodePath } from "@babel/traverse";

export class AttributeProcessor {
  classNameMap: CSSModuleExports;
  attrName: string;
  path: NodePath<t.JSXAttribute> | NodePath<t.CallExpression>;

  constructor(props: {
    attrName: string;
    classNameMap: CSSModuleExports;
    path: NodePath<t.JSXAttribute> | NodePath<t.CallExpression>;
  }) {
    this.attrName = props.attrName;
    this.classNameMap = props.classNameMap;
    this.path = props.path;
  }

  updateAttribute() {
    const node = (this.path as NodePath<t.JSXAttribute>).node;

    if (node.value?.type === "StringLiteral") {
      const transformedClassNames = this.processStringLiteral(node.value);
      node.value = t.stringLiteral(transformedClassNames);
    } else if (node.value?.type === "JSXExpressionContainer") {
      if (node.value.expression.type === "CallExpression") {
        if (node.value.expression.callee.type === "MemberExpression") {
          if (
            node.value.expression.callee.object.type === "ArrayExpression" &&
            node.value.expression.callee.property.type === "Identifier" &&
            node.value.expression.callee.property.name === "join"
          ) {
            const transformedClassNames = this.updateArrayExpression(
              node.value.expression.callee.object
            );
            node.value.expression.callee.object.elements =
              transformedClassNames;
          }
        } else {
          this.updateCallExpression(node.value.expression);
        }
      } else if (node.value.expression.type === "LogicalExpression") {
        const transformedNode = this.updateLogicalExpression(
          node.value.expression
        );
        node.value.expression = transformedNode;
      } else if (node.value.expression.type === "ConditionalExpression") {
        const transformedNode = this.updateConditionalExpression(
          node.value.expression
        );
        node.value.expression = transformedNode;
      } else if (node.value.expression.type === "Identifier") {
        this.updateIdentifier(node.value.expression);
      }
    }
  }

  updateIdentifier(node: t.Identifier) {
    const varName = node.name;
    const binding = this.path.scope.getBinding(varName);
    if (binding && t.isVariableDeclarator(binding.path.node)) {
      switch (binding.path.node.init?.type) {
        case "StringLiteral":
          const transformedClassNames = this.processStringLiteral(
            binding.path.node.init
          );
          binding.path.node.init.value = transformedClassNames;
          break;
        case "ObjectExpression":
          const properties = this.updateObjectExpression(
            binding.path.node.init
          );
          binding.path.node.init.properties = properties;
          break;
        case "ArrayExpression":
          const elements = this.updateArrayExpression(binding.path.node.init);
          binding.path.node.init.elements = elements;
          break;
        case "LogicalExpression":
          const transformedLogicalExpression = this.updateLogicalExpression(
            binding.path.node.init
          );
          binding.path.node.init = transformedLogicalExpression;
          break;
        case "ConditionalExpression":
          const transformedConditionalExpression =
            this.updateConditionalExpression(binding.path.node.init);
          binding.path.node.init = transformedConditionalExpression;
          break;
        default:
          break;
      }
    }
  }

  processStringLiteral = (node: t.StringLiteral) => {
    const classNames = node.value.split(" ");
    const transformedClassNames = classNames.map((className) => {
      const exportName = this.classNameMap[className]?.name;
      return exportName ?? className;
    });

    return transformedClassNames.join(" ");
  };

  updateObjectExpression = (node: t.ObjectExpression) => {
    const properties = node.properties.map((property) => {
      if (property.type === "ObjectProperty") {
        if (t.isStringLiteral(property.key)) {
          const exportName = this.classNameMap[property.key.value];
          if (exportName) {
            return t.objectProperty(
              t.stringLiteral(exportName.name),
              property.value
            );
          }
        } else if (t.isIdentifier(property.key)) {
          this.updateIdentifier(property.key);
          return property;
        }
      }
      return property;
    });
    return properties;
  };

  updateArrayExpression = (node: t.ArrayExpression) => {
    const elements: t.ArrayExpression["elements"] = node.elements.map(
      (element) => {
        switch (element?.type) {
          case "ObjectExpression":
            const properties = this.updateObjectExpression(element);
            element.properties = properties;
            return element;
          case "StringLiteral":
            const value = this.processStringLiteral(element);
            element.value = value;
            return element;
          case "ArrayExpression":
            const elements = this.updateArrayExpression(element);
            element.elements = elements;
            return element;
          case "LogicalExpression":
            return this.updateLogicalExpression(element);
          case "ConditionalExpression":
            return this.updateConditionalExpression(element);
          case "Identifier":
            this.updateIdentifier(element);
            return element;
          default:
            console.log(element);
            return element;
        }
      }
    );
    return elements;
  };

  updateLogicalExpression = (
    node: t.LogicalExpression
  ): t.LogicalExpression => {
    switch (node.right.type) {
      case "StringLiteral":
        const transformedClassNames = this.processStringLiteral(node.right);
        node.right.value = transformedClassNames;
        return node;
      case "ObjectExpression":
        const properties = this.updateObjectExpression(node.right);
        node.right.properties = properties;
        return node;
      case "ArrayExpression":
        const elements = this.updateArrayExpression(node.right);
        node.right.elements = elements;
        return node;
      case "ConditionalExpression":
        this.updateConditionalExpression(node.right);
        return node;
      case "CallExpression":
        this.updateCallExpression(node.right);
        return node;
      case "LogicalExpression":
        this.updateLogicalExpression(node.right);
        return node;
      default:
        return node;
    }
  };

  updateConditionalExpression = (node: t.ConditionalExpression) => {
    this.updateConditionalExpressionLeaf(node, "consequent");
    this.updateConditionalExpressionLeaf(node, "alternate");
    return node;
  };

  updateConditionalExpressionLeaf = (
    node: t.ConditionalExpression,
    type: "consequent" | "alternate"
  ) => {
    switch (node[type].type) {
      case "StringLiteral":
        const transformedClassNames = this.processStringLiteral(node[type]);
        node[type].value = transformedClassNames;
        return node;
      case "ObjectExpression":
        const properties = this.updateObjectExpression(node[type]);
        node[type].properties = properties;
        return node;
      case "ArrayExpression":
        const elements = this.updateArrayExpression(node[type]);
        node[type].elements = elements;
        return node;
      case "ConditionalExpression":
        return this.updateConditionalExpression(node[type]);
      case "LogicalExpression":
        return this.updateLogicalExpression(node[type]);
      case "CallExpression":
        return this.updateCallExpression(node[type]);
      default:
        return node;
    }
  };

  updateCallExpression = (node: t.CallExpression) => {
    const args = node.arguments.map((arg) => {
      if (arg.type === "StringLiteral") {
        const transformedClassNames = this.processStringLiteral(arg);
        arg.value = transformedClassNames;
        return arg;
      } else if (arg.type === "ObjectExpression") {
        const properties = this.updateObjectExpression(arg);
        arg.properties = properties;
      } else if (arg.type === "ArrayExpression") {
        const transformedClassNames = this.updateArrayExpression(arg);
        arg.elements = transformedClassNames;
      } else if (arg.type === "LogicalExpression") {
        const transformedClassNames = this.updateLogicalExpression(arg);
        arg.right = transformedClassNames;
      } else if (arg.type === "ConditionalExpression") {
        const transformedClassNames = this.updateConditionalExpression(arg);
        arg.consequent = transformedClassNames;
      } else if (arg.type === "CallExpression") {
        this.updateCallExpression(arg);
      }

      return arg;
    });

    node.arguments = args;
    return node;
  };
}
