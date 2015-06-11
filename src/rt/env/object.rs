use ::{JsResult, JsError};
use rt::{JsEnv, JsObject, JsArgs, JsValue, JsType, JsItem, JsStoreType, JsString};
use rt::{JsFnMode, JsDescriptor};
use rt::object::JsStoreKey;
use gc::*;
use syntax::Name;
use syntax::token::name;

// 15.2.1 The Object Constructor Called as a Function
// 15.2.2 The Object Constructor
pub fn Object_constructor(env: &mut JsEnv, mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	if mode.construct() {
		if args.argc > 0 {
			let arg = args.arg(env, 0);
			
			match arg.ty() {
				JsType::Object => Ok(arg),
				JsType::String | JsType::Boolean | JsType::Number => arg.to_object(env),
				_ => Ok(env.create_object().as_value(env))
			}
		} else {
			Ok(env.create_object().as_value(env))
		}
	} else {
		let arg = args.arg(env, 0);
		if arg.is_null_or_undefined() {
			Ok(env.create_object().as_value(env))
		} else {
			arg.to_object(env)
		}
	}
}

pub fn Object_create(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	assert!(args.argc <= 1);
	
	let mut result = JsObject::new_local(env, JsStoreType::Hash);
	
	if args.argc < 1 {
		return Err(JsError::new_type(env, ::errors::TYPE_INVALID));
	}

	result.set_prototype(env, Some(args.arg(env, 0)));
	
	Ok(result.as_value(env))
}

// 15.2.4.2 Object.prototype.toString ( )
pub fn Object_toString(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let this_arg = args.this(env);
	
	let result = if this_arg.is_undefined() {
		JsString::from_str(env, "[object Undefined]")
	} else if this_arg.is_null() {
		JsString::from_str(env, "[object Null]")
	} else {
		let mut result = String::new();
		
		result.push_str("[object ");
		
		let object = try!(this_arg.to_object(env));
		if let Some(class) = object.class(env) {
			result.push_str(&*env.ir.interner().get(class));
		} else {
			// TODO: Class should not be optional
			result.push_str("Unknown");
		}
		
		result.push_str("]");
		
		JsString::from_str(env, &result)
	};
	
	Ok(result.as_value(env))
}

// 15.2.4.3 Object.prototype.toLocaleString ( )
pub fn Object_toLocaleString(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let this = try!(args.this(env).to_object(env));
	let to_string = try!(this.get(env, name::TO_STRING));
	if to_string.is_callable(env) {
		to_string.call(env, this, Vec::new(), false)
	} else {
		Err(JsError::new_type(env, ::errors::TYPE_CANNOT_CALL_TO_STRING))
	}
}

// 15.2.4.4 Object.prototype.valueOf ( )
pub fn Object_valueOf(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	args.this(env).to_object(env)
}

// 15.2.4.5 Object.prototype.hasOwnProperty (V)
pub fn Object_hasOwnProperty(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let arg = args.arg(env, 0);
	
	let name = try!(env.intern_value(arg));
	
	let object = try!(args.this(env).to_object(env));
	
	let desc = object.get_own_property(env, name);
	
	Ok(env.new_bool(desc.is_some()))
}

// 15.2.4.6 Object.prototype.isPrototypeOf (V)
pub fn Object_isPrototypeOf(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let mut arg = args.arg(env, 0);
	
	let result = if arg.ty() != JsType::Object {
		false
	} else {
		let object = try!(args.this(env).to_object(env));
		let mut result = false;
		
		loop {
			if let Some(prototype) = arg.prototype(env) {
				arg = prototype;
				if object == arg {
					result = true;
					break;
				}
			} else {
				break;
			}
		}
		
		result
	};
	
	Ok(env.new_bool(result))
}

// 15.2.4.7 Object.prototype.propertyIsEnumerable (V)
pub fn Object_propertyIsEnumerable(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let name = args.arg(env, 0);
	let name = try!(env.intern_value(name));
	let object = try!(args.this(env).to_object(env));
	
	let result = if let Some(desc) = object.get_own_property(env, name) {
		desc.is_enumerable()
	} else {
		false
	};
	
	Ok(env.new_bool(result))
}

// 15.2.3.2 Object.getPrototypeOf ( O )
pub fn Object_getPrototypeOf(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let arg = args.arg(env, 0);
	
	if arg.ty() != JsType::Object {
		Err(JsError::new_type(env, ::errors::TYPE_INVALID))
	} else if let Some(prototype) = arg.prototype(env) {
		Ok(prototype)
	} else {
		Ok(env.new_undefined())
	}
}

// 15.2.3.6 Object.defineProperty ( O, P, Attributes )
pub fn Object_defineProperty(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let mut object = args.arg(env, 0);
	
	if object.ty() != JsType::Object {
		Err(JsError::new_type(env, ::errors::TYPE_INVALID))
	} else {
		let name = args.arg(env, 1);
		let name = try!(name.to_string(env));
		let name = env.ir.interner().intern(&name.to_string());
		
		let desc = args.arg(env, 2);
		let desc = try!(JsDescriptor::to_property_descriptor(env, desc));
		
		try!(object.define_own_property(env, name, desc, true));
		
		Ok(object)
	}
}

// http://ecma-international.org/ecma-262/5.1/#sec-15.2.3.3
pub fn Object_getOwnPropertyDescriptor(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let object = args.arg(env, 0);
	
	if object.ty() != JsType::Object {
		Err(JsError::new_type(env, ::errors::TYPE_INVALID))
	} else {
		let arg = args.arg(env, 1);
		let property = try!(arg.to_string(env)).to_string();
		let property = env.intern(&property);
		
		let property = object.get_own_property(env, property);
		
		if let Some(property) = property {
			property.from_property_descriptor(env)
		} else {
			Ok(env.new_undefined())
		}
	}
}

// 15.2.3.10 Object.preventExtensions ( O )
pub fn Object_preventExtensions(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let object = args.arg(env, 0);
	
	if object.ty() != JsType::Object {
		Err(JsError::new_type(env, ::errors::TYPE_INVALID))
	} else {
		object.unwrap_object(env).set_extensible(false);
		
		Ok(object)
	}
}

// 15.2.3.4 Object.getOwnPropertyNames ( O )
pub fn Object_getOwnPropertyNames(env: &mut JsEnv, _mode: JsFnMode, args: JsArgs) -> JsResult<Local<JsValue>> {
	let object = args.arg(env, 0);
	
	if object.ty() != JsType::Object {
		Err(JsError::new_type(env, ::errors::TYPE_INVALID))
	} else {
		let object = object.unwrap_object(env);
		let mut result = env.create_array();
		let mut offset = 0usize;
		
		for i in 0.. {
			match object.get_key(env, i) {
				JsStoreKey::Key(name, enumerable) => {
					if enumerable {
						let name = env.ir.interner().get(name);
						let name = JsString::from_str(env, &*name).as_value(env);
						try!(result.define_own_property(
							env,
							Name::from_index(offset),
							JsDescriptor::new_simple_value(name),
							false
						));
						
						offset += 1;
					}
				}
				JsStoreKey::End => break,
				JsStoreKey::Missing => {}
			}
		}
		
		Ok(result.as_value(env))
	}
}
