export interface Template {
    name: string;
    description: string;
    category?: string;
    version?: string;
}

export interface TemplateParam {
    name: string;
    description: string;
    type: string;
    required: boolean;
    default?: any;
    constraints?: {
        min?: number;
        max?: number;
        options?: string[];
    };
}

export interface TemplateSchema {
    name: string;
    description: string;
    params: TemplateParam[];
    version: string;
}
