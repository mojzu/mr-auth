/**
 * 
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { RequestFile } from './models';
import { RequestUserUpdateAccess } from './requestUserUpdateAccess';
import { RequestUserUpdatePassword } from './requestUserUpdatePassword';

export class RequestUserUpdate {
    'access'?: RequestUserUpdateAccess;
    'email'?: string;
    'enable'?: boolean;
    'id': string;
    'locale'?: string;
    'name'?: string;
    'password'?: RequestUserUpdatePassword;
    'timezone'?: string;

    static discriminator: string | undefined = undefined;

    static attributeTypeMap: Array<{name: string, baseName: string, type: string}> = [
        {
            "name": "access",
            "baseName": "access",
            "type": "RequestUserUpdateAccess"
        },
        {
            "name": "email",
            "baseName": "email",
            "type": "string"
        },
        {
            "name": "enable",
            "baseName": "enable",
            "type": "boolean"
        },
        {
            "name": "id",
            "baseName": "id",
            "type": "string"
        },
        {
            "name": "locale",
            "baseName": "locale",
            "type": "string"
        },
        {
            "name": "name",
            "baseName": "name",
            "type": "string"
        },
        {
            "name": "password",
            "baseName": "password",
            "type": "RequestUserUpdatePassword"
        },
        {
            "name": "timezone",
            "baseName": "timezone",
            "type": "string"
        }    ];

    static getAttributeTypeMap() {
        return RequestUserUpdate.attributeTypeMap;
    }
}

