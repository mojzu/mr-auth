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

export class ResponseUserOauth2Provider {
    'createdAt': Date;
    'oauth2Provider': string;
    '_static': boolean;
    'sub': string;
    'userId': string;

    static discriminator: string | undefined = undefined;

    static attributeTypeMap: Array<{name: string, baseName: string, type: string}> = [
        {
            "name": "createdAt",
            "baseName": "created_at",
            "type": "Date"
        },
        {
            "name": "oauth2Provider",
            "baseName": "oauth2_provider",
            "type": "string"
        },
        {
            "name": "_static",
            "baseName": "static",
            "type": "boolean"
        },
        {
            "name": "sub",
            "baseName": "sub",
            "type": "string"
        },
        {
            "name": "userId",
            "baseName": "user_id",
            "type": "string"
        }    ];

    static getAttributeTypeMap() {
        return ResponseUserOauth2Provider.attributeTypeMap;
    }
}

