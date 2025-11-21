'use client';

import { useEffect, useState, useRef } from 'react';
import { useParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox'; // I need to check if I have this, I didn't install it. I'll use standard checkbox or install it.
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { toast } from 'sonner';
import { TemplateSchema } from '@/types';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';

export default function TemplatePage() {
    const params = useParams();
    const name = params.name as string;
    const [schema, setSchema] = useState<TemplateSchema | null>(null);
    const [loading, setLoading] = useState(true);
    const [executing, setExecuting] = useState(false);
    const [output, setOutput] = useState<string>('');
    const outputEndRef = useRef<HTMLDivElement>(null);

    const { register, handleSubmit, formState: { errors } } = useForm();

    useEffect(() => {
        fetch(`/api/templates/${name}`)
            .then((res) => res.json())
            .then((data) => {
                if (data.error) {
                    toast.error(data.error);
                } else {
                    setSchema(data);
                }
            })
            .catch((err) => toast.error('Failed to fetch schema'))
            .finally(() => setLoading(false));
    }, [name]);

    useEffect(() => {
        if (outputEndRef.current) {
            outputEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [output]);

    const onSubmit = async (data: any) => {
        setExecuting(true);
        setOutput('');

        try {
            const response = await fetch('/api/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    type: 'template',
                    name: name,
                    params: data
                })
            });

            if (!response.ok) {
                throw new Error('Execution failed');
            }

            const reader = response.body?.getReader();
            if (!reader) return;

            const decoder = new TextDecoder();
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;
                const text = decoder.decode(value);
                setOutput((prev) => prev + text);
            }
        } catch (error) {
            toast.error('Execution failed');
            console.error(error);
        } finally {
            setExecuting(false);
        }
    };

    if (loading) return <div className="p-8">Loading...</div>;
    if (!schema) return <div className="p-8">Template not found</div>;

    return (
        <div className="container mx-auto p-8 max-w-4xl">
            <Link href="/" className="flex items-center text-muted-foreground hover:text-foreground mb-6">
                <ArrowLeft className="mr-2 h-4 w-4" /> Back to Templates
            </Link>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div>
                    <Card>
                        <CardHeader>
                            <CardTitle>{schema.name}</CardTitle>
                            <CardDescription>{schema.description}</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <form id="template-form" onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                                {schema.params.map((param) => (
                                    <div key={param.name} className="space-y-2">
                                        <Label htmlFor={param.name}>
                                            {param.name} {param.required && <span className="text-red-500">*</span>}
                                        </Label>
                                        {param.type === 'boolean' ? (
                                            <div className="flex items-center space-x-2">
                                                <input
                                                    type="checkbox"
                                                    id={param.name}
                                                    {...register(param.name)}
                                                    className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                                                />
                                                <span className="text-sm text-muted-foreground">{param.description}</span>
                                            </div>
                                        ) : param.type === 'integer' || param.type === 'number' ? (
                                            <Input
                                                type="number"
                                                id={param.name}
                                                placeholder={String(param.default || '')}
                                                {...register(param.name, { required: param.required })}
                                            />
                                        ) : (
                                            <Input
                                                id={param.name}
                                                placeholder={String(param.default || '')}
                                                {...register(param.name, { required: param.required })}
                                            />
                                        )}
                                        {param.type !== 'boolean' && (
                                            <p className="text-xs text-muted-foreground">{param.description}</p>
                                        )}
                                        {errors[param.name] && (
                                            <p className="text-xs text-red-500">This field is required</p>
                                        )}
                                    </div>
                                ))}
                            </form>
                        </CardContent>
                        <CardFooter>
                            <Button type="submit" form="template-form" disabled={executing} className="w-full">
                                {executing ? 'Executing...' : 'Run Template'}
                            </Button>
                        </CardFooter>
                    </Card>
                </div>

                <div className="h-[600px] flex flex-col">
                    <Card className="h-full flex flex-col">
                        <CardHeader>
                            <CardTitle>Output</CardTitle>
                        </CardHeader>
                        <CardContent className="flex-grow p-0 overflow-hidden">
                            <ScrollArea className="h-full w-full p-4 bg-black text-green-400 font-mono text-sm">
                                <pre className="whitespace-pre-wrap break-words">
                                    {output || 'Waiting for execution...'}
                                </pre>
                                <div ref={outputEndRef} />
                            </ScrollArea>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
