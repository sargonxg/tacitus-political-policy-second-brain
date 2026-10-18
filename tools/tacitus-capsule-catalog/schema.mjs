// SPDX-License-Identifier: Apache-2.0
// A deliberately limited interpreter for the repository's pinned capsule v1 schema.
export function validateSchema(value, schema, root = schema, path = '$') {
  const fail = message => { throw new Error(`${path}: ${message}`); };
  if (schema.$ref) {
    if (!schema.$ref.startsWith('#/$defs/')) fail('External references disabled');
    return validateSchema(value, root.$defs[schema.$ref.slice(8)], root, path);
  }
  if (schema.anyOf) {
    if (!schema.anyOf.some(s => { try {validateSchema(value,s,root,path);return true;} catch{return false;} })) fail('No allowed variant');
    return;
  }
  if ('const' in schema && value !== schema.const) fail('Unexpected constant');
  if (schema.enum && !schema.enum.includes(value)) fail('Unsupported value');
  const type = schema.type;
  if (type === 'null' && value !== null) fail('Expected null');
  if (type === 'integer' && !Number.isSafeInteger(value)) fail('Expected safe integer');
  if (type === 'boolean' && typeof value !== 'boolean') fail('Expected boolean');
  if (type === 'string' && typeof value !== 'string') fail('Expected string');
  if (schema.minimum !== undefined && value < schema.minimum) fail('Below minimum');
  if (schema.pattern && !new RegExp(schema.pattern).test(value)) fail('Invalid pattern');
  if (type === 'array') {
    if (!Array.isArray(value)) fail('Expected array');
    if (schema.maxItems !== undefined && value.length > schema.maxItems) fail('Too many items');
    value.forEach((v,i)=>validateSchema(v,schema.items,root,`${path}[${i}]`));
  }
  if (type === 'object') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) fail('Expected object');
    for (const k of schema.required ?? []) if (!Object.hasOwn(value,k)) fail(`Missing ${k}`);
    for (const [k,v] of Object.entries(value)) {
      if (Object.hasOwn(schema.properties ?? {},k)) validateSchema(v,schema.properties[k],root,`${path}.${k}`);
      else if (schema.additionalProperties === false) fail(`Unknown field ${k}`);
      else if (typeof schema.additionalProperties === 'object') validateSchema(v,schema.additionalProperties,root,`${path}.${k}`);
    }
  }
}
