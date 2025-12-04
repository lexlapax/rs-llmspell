'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Template } from '@/types';

export default function Dashboard() {
    const [templates, setTemplates] = useState<Template[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetch('/api/templates')
            .then((res) => res.json())
            .then((data) => {
                if (Array.isArray(data)) {
                    setTemplates(data);
                } else if (data && typeof data === 'object' && 'templates' in data && Array.isArray((data as { templates: Template[] }).templates)) {
                    setTemplates((data as { templates: Template[] }).templates);
                } else {
                    console.error('Invalid data format:', data);
                }
            })
            .catch((err) => console.error('Failed to fetch templates:', err))
            .finally(() => setLoading(false));
    }, []);

    if (loading) {
        return <div className="p-8">Loading templates...</div>;
    }

    return (
        <div className="container mx-auto p-8">
            <h1 className="text-3xl font-bold mb-8">Available Templates</h1>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {templates.map((template) => (
                    <Card key={template.name} className="flex flex-col">
                        <CardHeader>
                            <div className="flex justify-between items-start">
                                <CardTitle className="text-xl">{template.name}</CardTitle>
                                {template.category && <Badge>{template.category}</Badge>}
                            </div>
                            <CardDescription>{template.description}</CardDescription>
                        </CardHeader>
                        <CardContent className="flex-grow">
                            {/* Additional info if needed */}
                        </CardContent>
                        <CardFooter>
                            <Link href={`/templates/${template.id}`} className="w-full">
                                <Button className="w-full">Use Template</Button>
                            </Link>
                        </CardFooter>
                    </Card>
                ))}
            </div>
        </div>
    );
}
